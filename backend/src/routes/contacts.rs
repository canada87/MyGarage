use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use super::clean;
use crate::{
    error::{bad, ApiResult, AppError},
    models::{Contact, ContactInput},
    AppState,
};

#[derive(Deserialize)]
pub struct ListQuery {
    /// Contacts of this vehicle plus the global ones.
    vehicle_id: Option<i64>,
}

pub async fn list(State(state): State<AppState>, Query(q): Query<ListQuery>) -> ApiResult<Vec<Contact>> {
    let rows = sqlx::query_as(
        "SELECT * FROM contacts WHERE ?1 IS NULL OR vehicle_id = ?1 OR vehicle_id IS NULL
         ORDER BY vehicle_id IS NULL, category, name",
    )
    .bind(q.vehicle_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

fn validate(c: &ContactInput) -> Result<(), AppError> {
    if c.name.trim().is_empty() {
        return Err(bad("Il nome è obbligatorio"));
    }
    Ok(())
}

pub async fn create(State(state): State<AppState>, Json(c): Json<ContactInput>) -> ApiResult<Contact> {
    validate(&c)?;
    let row = sqlx::query_as(
        "INSERT INTO contacts (vehicle_id, name, category, phone, email, reference, notes)
         VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(c.vehicle_id)
    .bind(c.name.trim())
    .bind(&c.category)
    .bind(clean(c.phone))
    .bind(clean(c.email))
    .bind(clean(c.reference))
    .bind(clean(c.notes))
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn update(State(state): State<AppState>, Path(id): Path<i64>, Json(c): Json<ContactInput>) -> ApiResult<Contact> {
    validate(&c)?;
    let row = sqlx::query_as(
        "UPDATE contacts SET vehicle_id = ?, name = ?, category = ?, phone = ?, email = ?, reference = ?, notes = ?
         WHERE id = ? RETURNING *",
    )
    .bind(c.vehicle_id)
    .bind(c.name.trim())
    .bind(&c.category)
    .bind(clean(c.phone))
    .bind(clean(c.email))
    .bind(clean(c.reference))
    .bind(clean(c.notes))
    .bind(id)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    let res = sqlx::query("DELETE FROM contacts WHERE id = ?").bind(id).execute(&state.db).await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}
