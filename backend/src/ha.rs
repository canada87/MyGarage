//! Home Assistant integration: reads each car's odometer sensor and records it as a reading.
//!
//! At most one "ha" reading per car per day: later syncs on the same day update it. A value
//! lower than the current km is ignored (a sensor glitch must not rewind the car).

use std::time::Duration;

use anyhow::{anyhow, bail, Context};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::{due, settings::Settings};

const KM_PER_MILE: f64 = 1.609_344;

#[derive(Deserialize)]
struct HaState {
    entity_id: String,
    state: String,
    #[serde(default)]
    attributes: serde_json::Map<String, serde_json::Value>,
}

impl HaState {
    fn unit(&self) -> Option<&str> {
        self.attributes.get("unit_of_measurement").and_then(|v| v.as_str())
    }

    fn name(&self) -> String {
        self.attributes
            .get("friendly_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.entity_id)
            .to_string()
    }

    /// Odometer in km, converting from miles when needed.
    fn km(&self) -> anyhow::Result<i64> {
        let value: f64 = self
            .state
            .trim()
            .parse()
            .map_err(|_| anyhow!("il sensore {} vale \"{}\", non un numero", self.entity_id, self.state))?;
        let km = match self.unit().map(str::to_lowercase).as_deref() {
            Some("mi") => value * KM_PER_MILE,
            Some("m") => value / 1000.0,
            _ => value,
        };
        if !km.is_finite() || km < 0.0 {
            bail!("il sensore {} ha un valore non valido", self.entity_id);
        }
        Ok(km.round() as i64)
    }
}

pub struct Client {
    http: reqwest::Client,
    base: String,
    token: String,
}

impl Client {
    pub fn from_settings(s: &Settings) -> anyhow::Result<Client> {
        let base = s.ha_url.trim().trim_end_matches('/').to_string();
        if base.is_empty() || s.ha_token.is_empty() {
            bail!("Inserisci indirizzo e token di Home Assistant");
        }
        if !base.starts_with("http://") && !base.starts_with("https://") {
            bail!("L'indirizzo deve iniziare con http:// o https://");
        }
        let http = reqwest::Client::builder().timeout(Duration::from_secs(10)).build()?;
        Ok(Client { http, base, token: s.ha_token.clone() })
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> anyhow::Result<T> {
        let res = self
            .http
            .get(format!("{}{path}", self.base))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| anyhow!("Home Assistant non risponde ({})", root_cause(&e)))?;
        match res.status().as_u16() {
            200 => Ok(res.json().await.context("risposta di Home Assistant non valida")?),
            401 | 403 => bail!("token di Home Assistant non valido"),
            404 => bail!("non trovato su Home Assistant"),
            code => bail!("Home Assistant ha risposto con errore {code}"),
        }
    }

    /// Checks address and token; returns the Home Assistant message ("API running.").
    pub async fn ping(&self) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        struct Msg {
            message: String,
        }
        Ok(self.get::<Msg>("/api/").await?.message)
    }

    async fn state(&self, entity: &str) -> anyhow::Result<HaState> {
        self.get(&format!("/api/states/{entity}"))
            .await
            .map_err(|e| if e.to_string().starts_with("non trovato") { anyhow!("sensore {entity} non trovato") } else { e })
    }

    /// Sensors that look like an odometer: a distance in km or miles.
    pub async fn odometer_sensors(&self) -> anyhow::Result<Vec<Sensor>> {
        let states: Vec<HaState> = self.get("/api/states").await?;
        let mut out: Vec<Sensor> = states
            .into_iter()
            .filter(|s| s.entity_id.starts_with("sensor."))
            .filter(|s| {
                let distance = s.attributes.get("device_class").and_then(|v| v.as_str()) == Some("distance");
                let unit = matches!(s.unit().map(str::to_lowercase).as_deref(), Some("km" | "mi"));
                let named = ["odometer", "mileage", "chilometr", "contachilometri"].iter().any(|k| s.entity_id.contains(k));
                (distance || named) && unit
            })
            .map(|s| Sensor { name: s.name(), km: s.km().ok(), unit: s.unit().map(str::to_string), entity_id: s.entity_id })
            .collect();
        // Real odometers first: they have the largest values.
        out.sort_by(|a, b| b.km.cmp(&a.km));
        Ok(out)
    }
}

fn root_cause(e: &(dyn std::error::Error + 'static)) -> String {
    let mut cur = e;
    while let Some(next) = cur.source() {
        cur = next;
    }
    cur.to_string()
}

#[derive(Serialize)]
pub struct Sensor {
    pub entity_id: String,
    pub name: String,
    pub km: Option<i64>,
    pub unit: Option<String>,
}

#[derive(Serialize)]
pub struct SyncResult {
    pub vehicle_id: i64,
    pub vehicle: String,
    pub entity: String,
    pub km: Option<i64>,
    /// "nuova", "aggiornata", "invariata", "ignorata" or "errore".
    pub outcome: &'static str,
    pub message: String,
}

/// Reads every linked sensor and records the km. Errors per car end up in the results.
pub async fn sync(db: &SqlitePool, settings: &Settings) -> anyhow::Result<Vec<SyncResult>> {
    let cars: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id, name, ha_entity FROM vehicles WHERE archived = 0 AND ha_entity IS NOT NULL AND ha_entity != '' ORDER BY name",
    )
    .fetch_all(db)
    .await?;
    if cars.is_empty() {
        return Ok(vec![]);
    }
    let client = Client::from_settings(settings)?;
    let today = due::today();
    let mut out = Vec::new();
    for (vehicle_id, vehicle, entity) in cars {
        let mut r = SyncResult { vehicle_id, vehicle, entity: entity.clone(), km: None, outcome: "errore", message: String::new() };
        match client.state(&entity).await.and_then(|s| s.km()) {
            Err(e) => r.message = format!("{e:#}"),
            Ok(km) => {
                r.km = Some(km);
                let (current, _) = due::current_km(db, vehicle_id).await?;
                let today_ha: Option<(i64, i64)> = sqlx::query_as(
                    "SELECT id, km FROM odometer_readings WHERE vehicle_id = ? AND source = 'ha' AND date = ? ORDER BY id DESC LIMIT 1",
                )
                .bind(vehicle_id)
                .bind(today)
                .fetch_optional(db)
                .await?;
                if current.is_some_and(|c| km < c) {
                    r.outcome = "ignorata";
                    r.message = format!("il sensore segna {km} km, meno dei km attuali: lettura non salvata");
                } else if current == Some(km) {
                    r.outcome = "invariata";
                    r.message = "nessun km in più".into();
                } else if let Some((id, _)) = today_ha {
                    sqlx::query("UPDATE odometer_readings SET km = ? WHERE id = ?").bind(km).bind(id).execute(db).await?;
                    r.outcome = "aggiornata";
                    r.message = "lettura di oggi aggiornata".into();
                } else {
                    sqlx::query("INSERT INTO odometer_readings (vehicle_id, date, km, source) VALUES (?, ?, ?, 'ha')")
                        .bind(vehicle_id)
                        .bind(today)
                        .bind(km)
                        .execute(db)
                        .await?;
                    r.outcome = "nuova";
                    r.message = "nuova lettura salvata".into();
                }
            }
        }
        out.push(r);
    }
    Ok(out)
}

/// Runs a sync and stores a one-line summary shown in the settings page.
pub async fn sync_and_record(db: &SqlitePool) -> anyhow::Result<Vec<SyncResult>> {
    let settings = Settings::load(db).await?;
    if settings.ha_url.trim().is_empty() || settings.ha_token.is_empty() {
        return Ok(vec![]);
    }
    let result = sync(db, &settings).await;
    let summary = match &result {
        Ok(rs) if rs.is_empty() => "nessuna auto collegata a un sensore".to_string(),
        Ok(rs) => rs
            .iter()
            .map(|r| match r.km {
                Some(km) => format!("{}: {} km ({})", r.vehicle, km, r.outcome),
                None => format!("{}: {}", r.vehicle, r.message),
            })
            .collect::<Vec<_>>()
            .join(" · "),
        Err(e) => format!("errore: {e:#}"),
    };
    Settings::set(db, "ha_last_sync", &chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()).await?;
    Settings::set(db, "ha_last_result", &summary).await?;
    result
}

#[cfg(test)]
mod tests {
    use super::HaState;

    fn st(state: &str, unit: Option<&str>) -> HaState {
        let mut attributes = serde_json::Map::new();
        if let Some(u) = unit {
            attributes.insert("unit_of_measurement".into(), u.into());
        }
        HaState { entity_id: "sensor.car_odometer".into(), state: state.into(), attributes }
    }

    #[test]
    fn km_conversion() {
        assert_eq!(st("81872.0", Some("km")).km().unwrap(), 81872);
        assert_eq!(st("1000", Some("mi")).km().unwrap(), 1609);
        assert_eq!(st("12345", None).km().unwrap(), 12345);
        assert!(st("unavailable", Some("km")).km().is_err());
        assert!(st("-5", Some("km")).km().is_err());
    }
}
