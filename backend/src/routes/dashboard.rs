use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Datelike;
use serde::{Deserialize, Serialize};

use super::vehicles::{summaries, Counts, VehicleSummary};
use crate::{
    due::{self, ScheduleView, Status},
    error::ApiResult,
    models::Event,
    AppState,
};

#[derive(Serialize, sqlx::FromRow)]
pub struct CategoryTotal {
    pub category: String,
    pub total_cents: i64,
}

#[derive(Serialize)]
pub struct Dashboard {
    pub today: chrono::NaiveDate,
    pub counts: Counts,
    pub upcoming: Vec<ScheduleView>,
    pub vehicles: Vec<VehicleSummary>,
    pub year: i32,
    pub year_total_cents: i64,
    pub year_by_category: Vec<CategoryTotal>,
}

async fn totals_by_category(state: &AppState, year: i32, vehicle_id: Option<i64>) -> sqlx::Result<Vec<CategoryTotal>> {
    sqlx::query_as(
        "SELECT category, SUM(cost_cents) AS total_cents FROM events
         WHERE cost_cents IS NOT NULL AND strftime('%Y', date) = printf('%04d', ?1)
           AND (?2 IS NULL OR vehicle_id = ?2)
         GROUP BY category ORDER BY total_cents DESC",
    )
    .bind(year)
    .bind(vehicle_id)
    .fetch_all(&state.db)
    .await
}

pub async fn dashboard(State(state): State<AppState>) -> ApiResult<Dashboard> {
    let views = due::load_views(&state.db, None, false).await?;
    let counts = Counts::from_views(views.iter());
    let today = due::today();
    let year = today.year();
    // Everything that needs attention, then the next few that are fine.
    let mut upcoming: Vec<ScheduleView> = views.iter().filter(|v| v.status != Status::Ok).cloned().collect();
    upcoming.extend(views.iter().filter(|v| v.status == Status::Ok).take(5usize.saturating_sub(upcoming.len()).max(2)).cloned());
    let by_category = totals_by_category(&state, year, None).await?;
    Ok(Json(Dashboard {
        today,
        counts,
        upcoming,
        vehicles: summaries(&state, false).await?,
        year,
        year_total_cents: by_category.iter().map(|c| c.total_cents).sum(),
        year_by_category: by_category,
    }))
}

#[derive(Deserialize)]
pub struct ExpensesQuery {
    year: Option<i32>,
    vehicle_id: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct VehicleTotal {
    pub vehicle_id: i64,
    pub name: String,
    pub color: String,
    pub total_cents: i64,
}

#[derive(Serialize)]
pub struct Expenses {
    pub year: i32,
    pub years: Vec<i32>,
    pub total_cents: i64,
    pub by_category: Vec<CategoryTotal>,
    pub by_vehicle: Vec<VehicleTotal>,
    pub events: Vec<Event>,
}

pub async fn expenses(State(state): State<AppState>, Query(q): Query<ExpensesQuery>) -> ApiResult<Expenses> {
    let year = q.year.unwrap_or_else(|| due::today().year());
    let mut years: Vec<i32> = sqlx::query_scalar(
        "SELECT DISTINCT CAST(strftime('%Y', date) AS INTEGER) FROM events WHERE cost_cents IS NOT NULL ORDER BY 1 DESC",
    )
    .fetch_all(&state.db)
    .await?;
    if !years.contains(&year) {
        years.push(year);
        years.sort_unstable_by(|a, b| b.cmp(a));
    }
    let by_category = totals_by_category(&state, year, q.vehicle_id).await?;
    let by_vehicle = sqlx::query_as(
        "SELECT v.id AS vehicle_id, v.name, v.color, SUM(e.cost_cents) AS total_cents
         FROM events e JOIN vehicles v ON v.id = e.vehicle_id
         WHERE e.cost_cents IS NOT NULL AND strftime('%Y', e.date) = printf('%04d', ?1)
           AND (?2 IS NULL OR e.vehicle_id = ?2)
         GROUP BY v.id ORDER BY total_cents DESC",
    )
    .bind(year)
    .bind(q.vehicle_id)
    .fetch_all(&state.db)
    .await?;
    let events = sqlx::query_as(
        "SELECT * FROM events
         WHERE cost_cents IS NOT NULL AND strftime('%Y', date) = printf('%04d', ?1)
           AND (?2 IS NULL OR vehicle_id = ?2)
         ORDER BY date DESC, id DESC",
    )
    .bind(year)
    .bind(q.vehicle_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(Expenses {
        year,
        years,
        total_cents: by_category.iter().map(|c| c.total_cents).sum(),
        by_category,
        by_vehicle,
        events,
    }))
}
