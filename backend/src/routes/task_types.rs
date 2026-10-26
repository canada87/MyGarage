use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    error::{bad, ApiResult, AppError},
    models::{TaskType, TaskTypeInput},
    AppState,
};

pub async fn list(State(state): State<AppState>) -> ApiResult<Vec<TaskType>> {
    let rows = sqlx::query_as("SELECT * FROM task_types ORDER BY sort_order, name").fetch_all(&state.db).await?;
    Ok(Json(rows))
}

fn validate(t: &TaskTypeInput) -> Result<(), AppError> {
    if t.name.trim().is_empty() {
        return Err(bad("Il nome è obbligatorio"));
    }
    if t.anchor_month.is_some() != t.anchor_day.is_some() {
        return Err(bad("Per la scadenza annuale servono giorno e mese"));
    }
    Ok(())
}

pub async fn create(State(state): State<AppState>, Json(t): Json<TaskTypeInput>) -> ApiResult<TaskType> {
    validate(&t)?;
    let row = sqlx::query_as(
        "INSERT INTO task_types (name, category, icon, interval_months, interval_km, first_interval_months,
            anchor_month, anchor_day, suggested, sort_order)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(t.name.trim())
    .bind(&t.category)
    .bind(&t.icon)
    .bind(t.interval_months)
    .bind(t.interval_km)
    .bind(t.first_interval_months)
    .bind(t.anchor_month)
    .bind(t.anchor_day)
    .bind(t.suggested)
    .bind(t.sort_order.unwrap_or(200))
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn update(State(state): State<AppState>, Path(id): Path<i64>, Json(t): Json<TaskTypeInput>) -> ApiResult<TaskType> {
    validate(&t)?;
    let row = sqlx::query_as(
        "UPDATE task_types SET name = ?, category = ?, icon = ?, interval_months = ?, interval_km = ?,
            first_interval_months = ?, anchor_month = ?, anchor_day = ?, suggested = ?,
            sort_order = COALESCE(?, sort_order)
         WHERE id = ? RETURNING *",
    )
    .bind(t.name.trim())
    .bind(&t.category)
    .bind(&t.icon)
    .bind(t.interval_months)
    .bind(t.interval_km)
    .bind(t.first_interval_months)
    .bind(t.anchor_month)
    .bind(t.anchor_day)
    .bind(t.suggested)
    .bind(t.sort_order)
    .bind(id)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    let res = sqlx::query("DELETE FROM task_types WHERE id = ?").bind(id).execute(&state.db).await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}
