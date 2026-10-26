use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Vehicle {
    pub id: i64,
    pub name: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub plate: Option<String>,
    pub vin: Option<String>,
    pub year: Option<i64>,
    pub registration_date: Option<NaiveDate>,
    pub purchase_date: Option<NaiveDate>,
    pub purchased_used: bool,
    pub initial_km: i64,
    pub color: String,
    pub notes: Option<String>,
    pub archived: bool,
}

#[derive(Debug, Deserialize)]
pub struct VehicleInput {
    pub name: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub plate: Option<String>,
    pub vin: Option<String>,
    pub year: Option<i64>,
    pub registration_date: Option<NaiveDate>,
    pub purchase_date: Option<NaiveDate>,
    #[serde(default)]
    pub purchased_used: bool,
    pub color: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct TaskType {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub icon: String,
    pub interval_months: Option<i64>,
    pub interval_km: Option<i64>,
    pub first_interval_months: Option<i64>,
    pub anchor_month: Option<i64>,
    pub anchor_day: Option<i64>,
    pub suggested: bool,
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct TaskTypeInput {
    pub name: String,
    pub category: String,
    pub icon: String,
    pub interval_months: Option<i64>,
    pub interval_km: Option<i64>,
    pub first_interval_months: Option<i64>,
    pub anchor_month: Option<i64>,
    pub anchor_day: Option<i64>,
    #[serde(default)]
    pub suggested: bool,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Schedule {
    pub id: i64,
    pub vehicle_id: i64,
    pub task_type_id: Option<i64>,
    pub title: String,
    pub category: String,
    pub icon: String,
    pub interval_months: Option<i64>,
    pub interval_km: Option<i64>,
    pub anchor_month: Option<i64>,
    pub anchor_day: Option<i64>,
    pub baseline_date: Option<NaiveDate>,
    pub baseline_km: Option<i64>,
    pub manual_due_date: Option<NaiveDate>,
    pub manual_due_km: Option<i64>,
    pub manual_set_on: Option<NaiveDate>,
    pub last_done_date: Option<NaiveDate>,
    pub last_done_km: Option<i64>,
    pub next_due_date: Option<NaiveDate>,
    pub next_due_km: Option<i64>,
    pub warn_days: i64,
    pub warn_km: i64,
    pub email_enabled: bool,
    pub active: bool,
    pub notes: Option<String>,
}

/// How the user describes a deadline when creating or editing it.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DueMode {
    /// "Last time it was done on … / at … km" → next due is computed.
    Last,
    /// "Next due on … / at … km" → typed by hand.
    Next,
    /// Not known yet: the deadline is flagged as "to complete".
    Unknown,
    /// Leave baseline / manual values as they are (edit only).
    Keep,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleInput {
    pub vehicle_id: Option<i64>,
    pub task_type_id: Option<i64>,
    pub title: String,
    pub category: String,
    pub icon: String,
    pub interval_months: Option<i64>,
    pub interval_km: Option<i64>,
    pub anchor_month: Option<i64>,
    pub anchor_day: Option<i64>,
    pub warn_days: Option<i64>,
    pub warn_km: Option<i64>,
    #[serde(default = "default_true")]
    pub email_enabled: bool,
    #[serde(default = "default_true")]
    pub active: bool,
    pub notes: Option<String>,
    pub mode: DueMode,
    pub date: Option<NaiveDate>,
    pub km: Option<i64>,
}

pub fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Event {
    pub id: i64,
    pub vehicle_id: i64,
    pub schedule_id: Option<i64>,
    pub title: String,
    pub category: String,
    pub date: NaiveDate,
    pub km: Option<i64>,
    pub cost_cents: Option<i64>,
    pub provider: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EventInput {
    pub vehicle_id: i64,
    pub schedule_id: Option<i64>,
    pub title: String,
    pub category: String,
    pub date: NaiveDate,
    pub km: Option<i64>,
    pub cost_cents: Option<i64>,
    pub provider: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct OdometerReading {
    pub id: i64,
    pub vehicle_id: i64,
    pub event_id: Option<i64>,
    pub date: NaiveDate,
    pub km: i64,
    pub source: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OdometerInput {
    pub date: NaiveDate,
    pub km: i64,
    pub note: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Document {
    pub id: i64,
    pub vehicle_id: Option<i64>,
    pub event_id: Option<i64>,
    pub title: String,
    pub kind: String,
    pub stored_name: String,
    pub original_name: String,
    pub mime: String,
    pub size: i64,
    pub uploaded_at: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Contact {
    pub id: i64,
    pub vehicle_id: Option<i64>,
    pub name: String,
    pub category: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContactInput {
    pub vehicle_id: Option<i64>,
    pub name: String,
    pub category: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct TyreSet {
    pub id: i64,
    pub vehicle_id: i64,
    pub name: String,
    pub season: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub size: Option<String>,
    pub dot: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub initial_km: Option<i64>,
    pub notes: Option<String>,
    pub retired: bool,
}

#[derive(Debug, Deserialize)]
pub struct TyreSetInput {
    pub name: String,
    pub season: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub size: Option<String>,
    pub dot: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub initial_km: Option<i64>,
    pub notes: Option<String>,
    #[serde(default)]
    pub retired: bool,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct TyreMount {
    pub id: i64,
    pub tyre_set_id: i64,
    pub mounted_on: NaiveDate,
    pub mounted_km: Option<i64>,
    pub removed_on: Option<NaiveDate>,
    pub removed_km: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct TyreMountInput {
    pub mounted_on: NaiveDate,
    pub mounted_km: Option<i64>,
    pub removed_on: Option<NaiveDate>,
    pub removed_km: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DateKm {
    pub date: NaiveDate,
    pub km: Option<i64>,
}
