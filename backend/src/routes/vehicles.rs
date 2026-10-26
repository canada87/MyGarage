use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::clean;
use crate::{
    due::{self, ScheduleView, Status},
    error::{bad, ApiResult, AppError},
    models::{ScheduleInput, TaskType, Vehicle, VehicleInput},
    routes::schedules::insert_schedule,
    AppState,
};

#[derive(Serialize, Default)]
pub struct Counts {
    pub overdue: usize,
    pub soon: usize,
    pub ok: usize,
    pub setup: usize,
}

impl Counts {
    pub fn from_views<'a>(views: impl Iterator<Item = &'a ScheduleView>) -> Self {
        let mut c = Counts::default();
        for v in views {
            match v.status {
                Status::Overdue => c.overdue += 1,
                Status::Soon => c.soon += 1,
                Status::Ok => c.ok += 1,
                Status::Setup => c.setup += 1,
            }
        }
        c
    }
}

#[derive(Serialize)]
pub struct VehicleSummary {
    #[serde(flatten)]
    pub vehicle: Vehicle,
    pub current_km: Option<i64>,
    pub current_km_date: Option<NaiveDate>,
    pub counts: Counts,
    pub next: Option<ScheduleView>,
}

pub async fn summaries(state: &AppState, include_archived: bool) -> Result<Vec<VehicleSummary>, AppError> {
    let vehicles: Vec<Vehicle> = sqlx::query_as("SELECT * FROM vehicles WHERE archived = 0 OR ? ORDER BY archived, name")
        .bind(include_archived)
        .fetch_all(&state.db)
        .await?;
    let mut out = Vec::with_capacity(vehicles.len());
    for v in vehicles {
        let views = due::load_views(&state.db, Some(v.id), false).await?;
        let (km, km_date) = due::current_km(&state.db, v.id).await?;
        out.push(VehicleSummary {
            counts: Counts::from_views(views.iter()),
            next: views.iter().find(|s| s.status != Status::Setup).cloned(),
            current_km: km,
            current_km_date: km_date,
            vehicle: v,
        });
    }
    Ok(out)
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    archived: bool,
}

pub async fn list(State(state): State<AppState>, Query(q): Query<ListQuery>) -> ApiResult<Vec<VehicleSummary>> {
    Ok(Json(summaries(&state, q.archived).await?))
}

#[derive(Serialize)]
pub struct VehicleDetail {
    #[serde(flatten)]
    pub vehicle: Vehicle,
    pub current_km: Option<i64>,
    pub current_km_date: Option<NaiveDate>,
    pub counts: Counts,
    pub schedules: Vec<ScheduleView>,
}

pub async fn get(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<VehicleDetail> {
    let vehicle: Vehicle = sqlx::query_as("SELECT * FROM vehicles WHERE id = ?").bind(id).fetch_one(&state.db).await?;
    let schedules = due::load_views(&state.db, Some(id), true).await?;
    let (km, km_date) = due::current_km(&state.db, id).await?;
    Ok(Json(VehicleDetail {
        counts: Counts::from_views(schedules.iter().filter(|s| s.schedule.active)),
        vehicle,
        current_km: km,
        current_km_date: km_date,
        schedules,
    }))
}

/// One template picked in the "new car" wizard, with what the user knows about it.
#[derive(Deserialize)]
pub struct InitialSchedule {
    pub task_type_id: i64,
    pub mode: crate::models::DueMode,
    pub date: Option<NaiveDate>,
    pub km: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateVehicle {
    #[serde(flatten)]
    pub vehicle: VehicleInput,
    pub current_km: i64,
    pub current_km_date: Option<NaiveDate>,
    #[serde(default)]
    pub schedules: Vec<InitialSchedule>,
}

fn validate(v: &VehicleInput) -> Result<(), AppError> {
    if v.name.trim().is_empty() {
        return Err(bad("Il nome è obbligatorio"));
    }
    Ok(())
}

pub async fn create(State(state): State<AppState>, Json(input): Json<CreateVehicle>) -> ApiResult<Vehicle> {
    validate(&input.vehicle)?;
    if input.current_km < 0 {
        return Err(bad("I km non possono essere negativi"));
    }
    let v = input.vehicle;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO vehicles (name, make, model, plate, vin, year, registration_date, purchase_date, purchased_used, initial_km, color, notes)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(v.name.trim())
    .bind(clean(v.make))
    .bind(clean(v.model))
    .bind(clean(v.plate).map(|p| p.to_uppercase()))
    .bind(clean(v.vin))
    .bind(v.year)
    .bind(v.registration_date)
    .bind(v.purchase_date)
    .bind(v.purchased_used)
    .bind(input.current_km)
    .bind(clean(v.color).unwrap_or_else(|| "#2563EB".into()))
    .bind(clean(v.notes))
    .fetch_one(&state.db)
    .await?;

    sqlx::query("INSERT INTO odometer_readings (vehicle_id, date, km, source, note) VALUES (?, ?, ?, 'initial', 'Km all''inserimento')")
        .bind(id)
        .bind(input.current_km_date.unwrap_or_else(due::today))
        .bind(input.current_km)
        .execute(&state.db)
        .await?;

    for init in input.schedules {
        let tt: TaskType = sqlx::query_as("SELECT * FROM task_types WHERE id = ?")
            .bind(init.task_type_id)
            .fetch_one(&state.db)
            .await?;
        let s = ScheduleInput {
            vehicle_id: Some(id),
            task_type_id: Some(tt.id),
            title: tt.name,
            category: tt.category,
            icon: tt.icon,
            interval_months: tt.interval_months,
            interval_km: tt.interval_km,
            anchor_month: tt.anchor_month,
            anchor_day: tt.anchor_day,
            warn_days: None,
            warn_km: None,
            email_enabled: true,
            active: true,
            notes: None,
            mode: init.mode,
            date: init.date,
            km: init.km,
        };
        insert_schedule(&state, id, s).await?;
    }

    let vehicle = sqlx::query_as("SELECT * FROM vehicles WHERE id = ?").bind(id).fetch_one(&state.db).await?;
    Ok(Json(vehicle))
}

pub async fn update(State(state): State<AppState>, Path(id): Path<i64>, Json(v): Json<VehicleInput>) -> ApiResult<Vehicle> {
    validate(&v)?;
    let res = sqlx::query(
        "UPDATE vehicles SET name = ?, make = ?, model = ?, plate = ?, vin = ?, year = ?, registration_date = ?,
         purchase_date = ?, purchased_used = ?, color = ?, notes = ?, archived = ? WHERE id = ?",
    )
    .bind(v.name.trim())
    .bind(clean(v.make))
    .bind(clean(v.model))
    .bind(clean(v.plate).map(|p| p.to_uppercase()))
    .bind(clean(v.vin))
    .bind(v.year)
    .bind(v.registration_date)
    .bind(v.purchase_date)
    .bind(v.purchased_used)
    .bind(clean(v.color).unwrap_or_else(|| "#2563EB".into()))
    .bind(clean(v.notes))
    .bind(v.archived)
    .bind(id)
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    let vehicle = sqlx::query_as("SELECT * FROM vehicles WHERE id = ?").bind(id).fetch_one(&state.db).await?;
    Ok(Json(vehicle))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    let docs: Vec<(String,)> = sqlx::query_as("SELECT stored_name FROM documents WHERE vehicle_id = ?")
        .bind(id)
        .fetch_all(&state.db)
        .await?;
    let res = sqlx::query("DELETE FROM vehicles WHERE id = ?").bind(id).execute(&state.db).await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    for (name,) in docs {
        let _ = tokio::fs::remove_file(state.docs_dir.join(name)).await;
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}
