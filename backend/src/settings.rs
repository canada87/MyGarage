//! Key/value application settings stored in SQLite.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub email_enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    #[serde(skip_serializing)]
    pub smtp_password: String,
    pub email_from: String,
    /// Comma separated recipients.
    pub email_to: String,
    /// Local hour (0-23) of the daily check.
    pub send_hour: u32,
    pub default_warn_days: i64,
    pub default_warn_km: i64,
    #[serde(skip_serializing)]
    pub last_run_date: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            email_enabled: false,
            smtp_host: "smtp.gmail.com".into(),
            smtp_port: 587,
            smtp_username: String::new(),
            smtp_password: String::new(),
            email_from: String::new(),
            email_to: String::new(),
            send_hour: 8,
            default_warn_days: 30,
            default_warn_km: 2000,
            last_run_date: String::new(),
        }
    }
}

impl Settings {
    pub async fn load(db: &SqlitePool) -> sqlx::Result<Settings> {
        let rows: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM settings").fetch_all(db).await?;
        let map: HashMap<String, String> = rows.into_iter().collect();
        let d = Settings::default();
        let get = |k: &str| map.get(k).cloned();
        Ok(Settings {
            email_enabled: get("email_enabled").map_or(d.email_enabled, |v| v == "true"),
            smtp_host: get("smtp_host").unwrap_or(d.smtp_host),
            smtp_port: get("smtp_port").and_then(|v| v.parse().ok()).unwrap_or(d.smtp_port),
            smtp_username: get("smtp_username").unwrap_or(d.smtp_username),
            smtp_password: get("smtp_password").unwrap_or(d.smtp_password),
            email_from: get("email_from").unwrap_or(d.email_from),
            email_to: get("email_to").unwrap_or(d.email_to),
            send_hour: get("send_hour").and_then(|v| v.parse().ok()).unwrap_or(d.send_hour),
            default_warn_days: get("default_warn_days").and_then(|v| v.parse().ok()).unwrap_or(d.default_warn_days),
            default_warn_km: get("default_warn_km").and_then(|v| v.parse().ok()).unwrap_or(d.default_warn_km),
            last_run_date: get("last_run_date").unwrap_or(d.last_run_date),
        })
    }

    pub async fn set(db: &SqlitePool, key: &str, value: &str) -> sqlx::Result<()> {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
            .bind(key)
            .bind(value)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn save(&self, db: &SqlitePool) -> sqlx::Result<()> {
        let pairs = [
            ("email_enabled", self.email_enabled.to_string()),
            ("smtp_host", self.smtp_host.clone()),
            ("smtp_port", self.smtp_port.to_string()),
            ("smtp_username", self.smtp_username.clone()),
            ("smtp_password", self.smtp_password.clone()),
            ("email_from", self.email_from.clone()),
            ("email_to", self.email_to.clone()),
            ("send_hour", self.send_hour.to_string()),
            ("default_warn_days", self.default_warn_days.to_string()),
            ("default_warn_km", self.default_warn_km.to_string()),
        ];
        for (k, v) in pairs {
            Settings::set(db, k, &v).await?;
        }
        Ok(())
    }

    pub fn recipients(&self) -> Vec<String> {
        self.email_to.split([',', ';']).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    }

    /// Sender address: explicit `email_from`, otherwise the SMTP username (Gmail requires it anyway).
    pub fn sender(&self) -> String {
        if self.email_from.trim().is_empty() {
            self.smtp_username.clone()
        } else {
            self.email_from.clone()
        }
    }
}
