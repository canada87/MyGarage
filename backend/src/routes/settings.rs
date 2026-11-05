use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    error::{bad, ApiResult, AppError},
    ha, mailer,
    settings::Settings,
    AppState,
};

#[derive(Serialize)]
pub struct SettingsView {
    #[serde(flatten)]
    pub settings: Settings,
    pub password_set: bool,
    pub ha_token_set: bool,
}

#[derive(Deserialize)]
pub struct SettingsInput {
    pub email_enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    /// `None` keeps the stored password; an empty string clears it.
    pub smtp_password: Option<String>,
    pub email_from: String,
    pub email_to: String,
    pub send_hour: u32,
    pub default_warn_days: i64,
    pub default_warn_km: i64,
    #[serde(default)]
    pub ha_url: String,
    /// `None` keeps the stored token; an empty string clears it.
    pub ha_token: Option<String>,
}

fn view(s: Settings) -> SettingsView {
    SettingsView { password_set: !s.smtp_password.is_empty(), ha_token_set: !s.ha_token.is_empty(), settings: s }
}

pub async fn get(State(state): State<AppState>) -> ApiResult<SettingsView> {
    Ok(Json(view(Settings::load(&state.db).await?)))
}

pub async fn update(State(state): State<AppState>, Json(i): Json<SettingsInput>) -> ApiResult<SettingsView> {
    if i.send_hour > 23 {
        return Err(bad("L'ora di invio deve essere tra 0 e 23"));
    }
    if i.default_warn_days < 0 || i.default_warn_km < 0 {
        return Err(bad("I preavvisi non possono essere negativi"));
    }
    let mut s = Settings::load(&state.db).await?;
    s.email_enabled = i.email_enabled;
    s.smtp_host = i.smtp_host.trim().to_string();
    s.smtp_port = i.smtp_port;
    s.smtp_username = i.smtp_username.trim().to_string();
    if let Some(p) = i.smtp_password {
        // Gmail shows app passwords in groups of four: spaces are not part of it.
        s.smtp_password = p.split_whitespace().collect();
    }
    s.email_from = i.email_from.trim().to_string();
    s.email_to = i.email_to.trim().to_string();
    s.send_hour = i.send_hour;
    s.default_warn_days = i.default_warn_days;
    s.default_warn_km = i.default_warn_km;
    s.ha_url = i.ha_url.trim().trim_end_matches('/').to_string();
    if let Some(t) = i.ha_token {
        s.ha_token = t.trim().to_string();
    }
    s.save(&state.db).await?;
    Ok(Json(view(s)))
}

pub async fn test_email(State(state): State<AppState>) -> ApiResult<serde_json::Value> {
    let s = Settings::load(&state.db).await?;
    mailer::send_test(&s, &state.base_url).await.map_err(|e| AppError::BadRequest(format!("Invio non riuscito: {e:#}")))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Checks the Home Assistant address and token.
pub async fn test_ha(State(state): State<AppState>) -> ApiResult<serde_json::Value> {
    let s = Settings::load(&state.db).await?;
    let client = ha::Client::from_settings(&s).map_err(|e| AppError::BadRequest(e.to_string()))?;
    let msg = client.ping().await.map_err(|e| AppError::BadRequest(format!("{e:#}")))?;
    Ok(Json(serde_json::json!({ "ok": true, "message": msg })))
}

/// Sensors that look like an odometer, to pick one for a car.
pub async fn ha_sensors(State(state): State<AppState>) -> ApiResult<Vec<ha::Sensor>> {
    let s = Settings::load(&state.db).await?;
    let client = ha::Client::from_settings(&s).map_err(|e| AppError::BadRequest(e.to_string()))?;
    let sensors = client.odometer_sensors().await.map_err(|e| AppError::BadRequest(format!("{e:#}")))?;
    Ok(Json(sensors))
}

/// Reads the linked sensors now.
pub async fn ha_sync(State(state): State<AppState>) -> ApiResult<Vec<ha::SyncResult>> {
    let results = ha::sync_and_record(&state.db).await.map_err(|e| AppError::BadRequest(format!("{e:#}")))?;
    Ok(Json(results))
}
