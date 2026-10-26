use axum::{
    extract::{Path, State},
    Json,
};

use super::clean;
use crate::{
    error::{bad, ApiResult, AppError},
    models::{OdometerInput, OdometerReading},
    AppState,
};

pub async fn list(State(state): State<AppState>, Path(vehicle_id): Path<i64>) -> ApiResult<Vec<OdometerReading>> {
    let rows = sqlx::query_as("SELECT * FROM odometer_readings WHERE vehicle_id = ? ORDER BY date DESC, km DESC")
        .bind(vehicle_id)
        .fetch_all(&state.db)
        .await?;
    Ok(Json(rows))
}

fn validate(r: &OdometerInput) -> Result<(), AppError> {
    if r.km < 0 {
        return Err(bad("I km non possono essere negativi"));
    }
    Ok(())
}

pub async fn create(
    State(state): State<AppState>,
    Path(vehicle_id): Path<i64>,
    Json(r): Json<OdometerInput>,
) -> ApiResult<OdometerReading> {
    validate(&r)?;
    let source = match r.source.as_deref() {
        Some("ha") => "ha",
        _ => "manual",
    };
    let row = sqlx::query_as(
        "INSERT INTO odometer_readings (vehicle_id, date, km, source, note) VALUES (?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(vehicle_id)
    .bind(r.date)
    .bind(r.km)
    .bind(source)
    .bind(clean(r.note))
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn update(State(state): State<AppState>, Path(id): Path<i64>, Json(r): Json<OdometerInput>) -> ApiResult<OdometerReading> {
    validate(&r)?;
    let linked: Option<i64> = sqlx::query_scalar("SELECT event_id FROM odometer_readings WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    if linked.is_some() {
        return Err(bad("Questa lettura viene da un intervento: modifica l'intervento"));
    }
    let row = sqlx::query_as("UPDATE odometer_readings SET date = ?, km = ?, note = ? WHERE id = ? RETURNING *")
        .bind(r.date)
        .bind(r.km)
        .bind(clean(r.note))
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    Ok(Json(row))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    let row: (Option<i64>, i64) = sqlx::query_as("SELECT event_id, vehicle_id FROM odometer_readings WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    if row.0.is_some() {
        return Err(bad("Questa lettura viene da un intervento: modifica l'intervento"));
    }
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM odometer_readings WHERE vehicle_id = ?")
        .bind(row.1)
        .fetch_one(&state.db)
        .await?;
    if remaining <= 1 {
        return Err(AppError::BadRequest("Serve almeno una lettura dei km".into()));
    }
    sqlx::query("DELETE FROM odometer_readings WHERE id = ?").bind(id).execute(&state.db).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
