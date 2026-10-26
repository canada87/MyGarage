use axum::{
    extract::{Path, State},
    Json,
};
use chrono::NaiveDate;
use serde::Serialize;

use super::clean;
use crate::{
    error::{bad, ApiResult, AppError},
    models::{DateKm, TyreMount, TyreMountInput, TyreSet, TyreSetInput},
    AppState,
};

#[derive(Serialize)]
pub struct MountView {
    #[serde(flatten)]
    pub mount: TyreMount,
    pub km_start: Option<i64>,
    pub km_end: Option<i64>,
    pub distance: Option<i64>,
    /// True when a start/end km was derived from odometer readings.
    pub estimated: bool,
}

#[derive(Serialize)]
pub struct TyreSetView {
    #[serde(flatten)]
    pub set: TyreSet,
    pub mounted: bool,
    pub total_km: i64,
    pub estimated: bool,
    pub mounts: Vec<MountView>,
}

/// Odometer value at `date`, linearly interpolated between the surrounding readings.
/// Never extrapolates beyond the first/last reading.
fn km_at(readings: &[(NaiveDate, i64)], date: NaiveDate) -> Option<i64> {
    let before = readings.iter().filter(|(d, _)| *d <= date).max_by_key(|(d, k)| (*d, *k));
    let after = readings.iter().filter(|(d, _)| *d >= date).min_by_key(|(d, k)| (*d, *k));
    match (before, after) {
        (Some(&(d1, k1)), Some(&(d2, k2))) if d2 > d1 => {
            let span = (d2 - d1).num_days() as f64;
            let part = (date - d1).num_days() as f64;
            Some(k1 + ((k2 - k1) as f64 * part / span).round() as i64)
        }
        (Some(&(_, k)), _) => Some(k),
        (None, Some(&(_, k))) => Some(k),
        (None, None) => None,
    }
}

pub fn build_views(sets: Vec<TyreSet>, mounts: Vec<TyreMount>, readings: &[(NaiveDate, i64)]) -> Vec<TyreSetView> {
    let current_km = readings.iter().max_by_key(|(d, k)| (*d, *k)).map(|(_, k)| *k);
    sets.into_iter()
        .map(|set| {
            let mut estimated = false;
            let mut total = set.initial_km.unwrap_or(0);
            let mut mounted = false;
            let mut views = Vec::new();
            for m in mounts.iter().filter(|m| m.tyre_set_id == set.id) {
                let mut est = false;
                let km_start = m.mounted_km.or_else(|| {
                    est = true;
                    km_at(readings, m.mounted_on)
                });
                let km_end = match (m.removed_km, m.removed_on) {
                    (Some(k), _) => Some(k),
                    (None, Some(d)) => {
                        est = true;
                        km_at(readings, d)
                    }
                    (None, None) => {
                        mounted = true;
                        current_km
                    }
                };
                let distance = match (km_start, km_end) {
                    (Some(a), Some(b)) => Some((b - a).max(0)),
                    _ => None,
                };
                total += distance.unwrap_or(0);
                estimated |= est;
                views.push(MountView { mount: m.clone(), km_start, km_end, distance, estimated: est });
            }
            TyreSetView { set, mounted, total_km: total, estimated, mounts: views }
        })
        .collect()
}

pub async fn list(State(state): State<AppState>, Path(vehicle_id): Path<i64>) -> ApiResult<Vec<TyreSetView>> {
    let sets: Vec<TyreSet> = sqlx::query_as("SELECT * FROM tyre_sets WHERE vehicle_id = ? ORDER BY retired, id")
        .bind(vehicle_id)
        .fetch_all(&state.db)
        .await?;
    let mounts: Vec<TyreMount> = sqlx::query_as(
        "SELECT m.* FROM tyre_mounts m JOIN tyre_sets s ON s.id = m.tyre_set_id WHERE s.vehicle_id = ? ORDER BY m.mounted_on DESC",
    )
    .bind(vehicle_id)
    .fetch_all(&state.db)
    .await?;
    let readings: Vec<(NaiveDate, i64)> = sqlx::query_as("SELECT date, km FROM odometer_readings WHERE vehicle_id = ?")
        .bind(vehicle_id)
        .fetch_all(&state.db)
        .await?;
    let mut views = build_views(sets, mounts, &readings);
    views.sort_by_key(|v| (!v.mounted, v.set.retired, v.set.id));
    Ok(Json(views))
}

fn validate(t: &TyreSetInput) -> Result<(), AppError> {
    if t.name.trim().is_empty() {
        return Err(bad("Dai un nome al treno di gomme"));
    }
    Ok(())
}

pub async fn create(State(state): State<AppState>, Path(vehicle_id): Path<i64>, Json(t): Json<TyreSetInput>) -> ApiResult<TyreSet> {
    validate(&t)?;
    let row = sqlx::query_as(
        "INSERT INTO tyre_sets (vehicle_id, name, season, brand, model, size, dot, purchase_date, initial_km, notes, retired)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(vehicle_id)
    .bind(t.name.trim())
    .bind(clean(t.season))
    .bind(clean(t.brand))
    .bind(clean(t.model))
    .bind(clean(t.size))
    .bind(clean(t.dot))
    .bind(t.purchase_date)
    .bind(t.initial_km)
    .bind(clean(t.notes))
    .bind(t.retired)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn update(State(state): State<AppState>, Path(id): Path<i64>, Json(t): Json<TyreSetInput>) -> ApiResult<TyreSet> {
    validate(&t)?;
    let row = sqlx::query_as(
        "UPDATE tyre_sets SET name = ?, season = ?, brand = ?, model = ?, size = ?, dot = ?, purchase_date = ?,
            initial_km = ?, notes = ?, retired = ?
         WHERE id = ? RETURNING *",
    )
    .bind(t.name.trim())
    .bind(clean(t.season))
    .bind(clean(t.brand))
    .bind(clean(t.model))
    .bind(clean(t.size))
    .bind(clean(t.dot))
    .bind(t.purchase_date)
    .bind(t.initial_km)
    .bind(clean(t.notes))
    .bind(t.retired)
    .bind(id)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    let res = sqlx::query("DELETE FROM tyre_sets WHERE id = ?").bind(id).execute(&state.db).await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Mounts a set; whatever set was mounted on the same car is removed at the same date/km.
pub async fn mount(State(state): State<AppState>, Path(id): Path<i64>, Json(p): Json<DateKm>) -> ApiResult<TyreMount> {
    let vehicle_id: i64 = sqlx::query_scalar("SELECT vehicle_id FROM tyre_sets WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    sqlx::query(
        "UPDATE tyre_mounts SET removed_on = ?, removed_km = ?
         WHERE removed_on IS NULL AND tyre_set_id IN (SELECT id FROM tyre_sets WHERE vehicle_id = ?)",
    )
    .bind(p.date)
    .bind(p.km)
    .bind(vehicle_id)
    .execute(&state.db)
    .await?;
    let row = sqlx::query_as("INSERT INTO tyre_mounts (tyre_set_id, mounted_on, mounted_km) VALUES (?, ?, ?) RETURNING *")
        .bind(id)
        .bind(p.date)
        .bind(p.km)
        .fetch_one(&state.db)
        .await?;
    Ok(Json(row))
}

pub async fn unmount(State(state): State<AppState>, Path(id): Path<i64>, Json(p): Json<DateKm>) -> ApiResult<serde_json::Value> {
    sqlx::query("UPDATE tyre_mounts SET removed_on = ?, removed_km = ? WHERE tyre_set_id = ? AND removed_on IS NULL")
        .bind(p.date)
        .bind(p.km)
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn update_mount(State(state): State<AppState>, Path(id): Path<i64>, Json(m): Json<TyreMountInput>) -> ApiResult<TyreMount> {
    if m.removed_on.is_some_and(|r| r < m.mounted_on) {
        return Err(bad("Lo smontaggio non può precedere il montaggio"));
    }
    let row = sqlx::query_as(
        "UPDATE tyre_mounts SET mounted_on = ?, mounted_km = ?, removed_on = ?, removed_km = ? WHERE id = ? RETURNING *",
    )
    .bind(m.mounted_on)
    .bind(m.mounted_km)
    .bind(m.removed_on)
    .bind(m.removed_km)
    .bind(id)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn delete_mount(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    let res = sqlx::query("DELETE FROM tyre_mounts WHERE id = ?").bind(id).execute(&state.db).await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn interpolates_missing_km() {
        let readings = vec![(d("2026-01-01"), 10000), (d("2026-01-31"), 13000)];
        assert_eq!(km_at(&readings, d("2026-01-16")), Some(11500));
        assert_eq!(km_at(&readings, d("2025-06-01")), Some(10000));
        assert_eq!(km_at(&readings, d("2026-06-01")), Some(13000));
    }

    #[test]
    fn totals_per_set() {
        let set = TyreSet {
            id: 1,
            vehicle_id: 1,
            name: "Estive".into(),
            season: None,
            brand: None,
            model: None,
            size: None,
            dot: None,
            purchase_date: None,
            initial_km: Some(500),
            notes: None,
            retired: false,
        };
        let mounts = vec![
            TyreMount { id: 1, tyre_set_id: 1, mounted_on: d("2025-04-10"), mounted_km: Some(1000), removed_on: Some(d("2025-11-10")), removed_km: None },
            TyreMount { id: 2, tyre_set_id: 1, mounted_on: d("2026-04-10"), mounted_km: Some(9000), removed_on: None, removed_km: None },
        ];
        let readings = vec![(d("2025-01-01"), 0), (d("2025-11-10"), 8000), (d("2026-10-01"), 15000)];
        let v = build_views(vec![set], mounts, &readings);
        // 500 initial + 7000 (first season) + 6000 (current)
        assert_eq!(v[0].total_km, 13500);
        assert!(v[0].mounted);
        assert!(v[0].estimated);
    }
}
