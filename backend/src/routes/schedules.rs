use axum::{
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use super::clean;
use crate::{
    due::{self, ScheduleView},
    error::{bad, ApiResult, AppError},
    ics,
    models::{DueMode, Schedule, ScheduleInput},
    settings::Settings,
    AppState,
};

fn validate(s: &ScheduleInput) -> Result<(), AppError> {
    if s.title.trim().is_empty() {
        return Err(bad("Il titolo è obbligatorio"));
    }
    if s.anchor_month.is_some() != s.anchor_day.is_some() {
        return Err(bad("Per la scadenza annuale servono giorno e mese"));
    }
    if let (Some(m), Some(d)) = (s.anchor_month, s.anchor_day) {
        if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
            return Err(bad("Giorno o mese non validi"));
        }
    }
    if s.interval_months.is_some_and(|m| m <= 0) || s.interval_km.is_some_and(|k| k <= 0) {
        return Err(bad("Gli intervalli devono essere positivi"));
    }
    if s.mode != DueMode::Keep && s.mode != DueMode::Unknown && s.date.is_none() && s.km.is_none() {
        return Err(bad("Indica una data o dei km, oppure scegli \"Non lo so\""));
    }
    Ok(())
}

pub async fn insert_schedule(state: &AppState, vehicle_id: i64, s: ScheduleInput) -> Result<i64, AppError> {
    validate(&s)?;
    let defaults = Settings::load(&state.db).await?;
    let (baseline_date, baseline_km, manual_date, manual_km, manual_on) = match s.mode {
        DueMode::Last => (s.date, s.km, None, None, None),
        DueMode::Next => (None, None, s.date, s.km, Some(due::today())),
        DueMode::Unknown | DueMode::Keep => (None, None, None, None, None),
    };
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO schedules (vehicle_id, task_type_id, title, category, icon, interval_months, interval_km,
            anchor_month, anchor_day, baseline_date, baseline_km, manual_due_date, manual_due_km, manual_set_on,
            warn_days, warn_km, email_enabled, active, notes)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(vehicle_id)
    .bind(s.task_type_id)
    .bind(s.title.trim())
    .bind(&s.category)
    .bind(&s.icon)
    .bind(s.interval_months)
    .bind(s.interval_km)
    .bind(s.anchor_month)
    .bind(s.anchor_day)
    .bind(baseline_date)
    .bind(baseline_km)
    .bind(manual_date)
    .bind(manual_km)
    .bind(manual_on)
    .bind(s.warn_days.unwrap_or(defaults.default_warn_days))
    .bind(s.warn_km.unwrap_or(defaults.default_warn_km))
    .bind(s.email_enabled)
    .bind(s.active)
    .bind(clean(s.notes))
    .fetch_one(&state.db)
    .await?;
    due::recompute(&state.db, id).await?;
    Ok(id)
}

async fn view(state: &AppState, id: i64) -> Result<ScheduleView, AppError> {
    let vehicle_id: i64 = sqlx::query_scalar("SELECT vehicle_id FROM schedules WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    due::load_views(&state.db, Some(vehicle_id), true)
        .await?
        .into_iter()
        .find(|v| v.schedule.id == id)
        .ok_or(AppError::NotFound)
}

#[derive(Deserialize)]
pub struct ListQuery {
    vehicle_id: Option<i64>,
    #[serde(default)]
    include_inactive: bool,
}

pub async fn list(State(state): State<AppState>, Query(q): Query<ListQuery>) -> ApiResult<Vec<ScheduleView>> {
    Ok(Json(due::load_views(&state.db, q.vehicle_id, q.include_inactive).await?))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<ScheduleView> {
    Ok(Json(view(&state, id).await?))
}

pub async fn create(State(state): State<AppState>, Json(s): Json<ScheduleInput>) -> ApiResult<ScheduleView> {
    let vehicle_id = s.vehicle_id.ok_or_else(|| bad("Auto mancante"))?;
    let id = insert_schedule(&state, vehicle_id, s).await?;
    Ok(Json(view(&state, id).await?))
}

pub async fn update(State(state): State<AppState>, Path(id): Path<i64>, Json(s): Json<ScheduleInput>) -> ApiResult<ScheduleView> {
    validate(&s)?;
    let old: Schedule = sqlx::query_as("SELECT * FROM schedules WHERE id = ?").bind(id).fetch_one(&state.db).await?;
    let (baseline_date, baseline_km, manual_date, manual_km, manual_on) = match s.mode {
        DueMode::Last => (s.date, s.km, None, None, None),
        DueMode::Next => (old.baseline_date, old.baseline_km, s.date, s.km, Some(due::today())),
        DueMode::Unknown => (None, None, None, None, None),
        DueMode::Keep => (old.baseline_date, old.baseline_km, old.manual_due_date, old.manual_due_km, old.manual_set_on),
    };
    // A changed yearly anchor invalidates the cached deadline.
    let anchor_changed = old.anchor_month != s.anchor_month || old.anchor_day != s.anchor_day;
    let next_due_date = if anchor_changed || s.mode != DueMode::Keep { None } else { old.next_due_date };
    sqlx::query(
        "UPDATE schedules SET title = ?, category = ?, icon = ?, interval_months = ?, interval_km = ?,
            anchor_month = ?, anchor_day = ?, baseline_date = ?, baseline_km = ?, manual_due_date = ?,
            manual_due_km = ?, manual_set_on = ?, next_due_date = ?, warn_days = ?, warn_km = ?,
            email_enabled = ?, active = ?, notes = ?
         WHERE id = ?",
    )
    .bind(s.title.trim())
    .bind(&s.category)
    .bind(&s.icon)
    .bind(s.interval_months)
    .bind(s.interval_km)
    .bind(s.anchor_month)
    .bind(s.anchor_day)
    .bind(baseline_date)
    .bind(baseline_km)
    .bind(manual_date)
    .bind(manual_km)
    .bind(manual_on)
    .bind(next_due_date)
    .bind(s.warn_days.unwrap_or(old.warn_days))
    .bind(s.warn_km.unwrap_or(old.warn_km))
    .bind(s.email_enabled)
    .bind(s.active)
    .bind(clean(s.notes))
    .bind(id)
    .execute(&state.db)
    .await?;
    due::recompute(&state.db, id).await?;
    Ok(Json(view(&state, id).await?))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    let res = sqlx::query("DELETE FROM schedules WHERE id = ?").bind(id).execute(&state.db).await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn ics(State(state): State<AppState>, Path(id): Path<i64>) -> Result<impl IntoResponse, AppError> {
    let v = view(&state, id).await?;
    let Some(date) = v.schedule.next_due_date else {
        return Err(bad("Questa scadenza non ha una data"));
    };
    let body = ics::event(&v, date, &state.base_url);
    Ok((
        [
            (header::CONTENT_TYPE, "text/calendar; charset=utf-8".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"scadenza-{id}.ics\"")),
        ],
        body,
    ))
}
