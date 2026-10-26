//! Deadline computation: when a schedule is next due and how urgent it is.

use chrono::{Datelike, Local, Months, NaiveDate};
use serde::Serialize;
use sqlx::SqlitePool;

use crate::models::Schedule;

/// Readings older than this make km-based deadlines unreliable.
pub const KM_STALE_DAYS: i64 = 45;
/// After an anchored task is done, the same season's deadline is considered covered.
const ANCHOR_GRACE_DAYS: i64 = 60;

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

pub fn add_months(d: NaiveDate, months: i64) -> NaiveDate {
    if months >= 0 {
        d.checked_add_months(Months::new(months as u32)).unwrap_or(d)
    } else {
        d.checked_sub_months(Months::new((-months) as u32)).unwrap_or(d)
    }
}

fn anchor_date(year: i32, month: u32, day: u32) -> NaiveDate {
    // Clamp the day for short months (e.g. 31/04 → 30/04).
    (0..4)
        .find_map(|back| NaiveDate::from_ymd_opt(year, month, day - back))
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 1).unwrap())
}

/// First yearly occurrence of `month/day` strictly after `after + grace`.
fn next_anchor(after: NaiveDate, month: u32, day: u32) -> NaiveDate {
    let threshold = after + chrono::Duration::days(ANCHOR_GRACE_DAYS);
    let mut year = threshold.year();
    loop {
        let d = anchor_date(year, month, day);
        if d > threshold {
            return d;
        }
        year += 1;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Computed {
    pub last_date: Option<NaiveDate>,
    pub last_km: Option<i64>,
    pub next_date: Option<NaiveDate>,
    pub next_km: Option<i64>,
}

/// `last_event` is the most recent event linked to the schedule: (date, km).
pub fn compute(s: &Schedule, last_event: Option<(NaiveDate, Option<i64>)>, today: NaiveDate) -> Computed {
    let baseline = (s.baseline_date.is_some() || s.baseline_km.is_some()).then_some((s.baseline_date, s.baseline_km));
    let (last_date, last_km) = match (last_event, baseline) {
        (Some((ed, ek)), Some((Some(bd), bk))) if bd > ed => (Some(bd), bk.or(ek)),
        (Some((ed, ek)), _) => (Some(ed), ek),
        (None, Some((bd, bk))) => (bd, bk),
        (None, None) => (None, None),
    };

    let manual_active = (s.manual_due_date.is_some() || s.manual_due_km.is_some())
        && match (last_event, s.manual_set_on) {
            (Some((ed, _)), Some(set_on)) => ed < set_on,
            (Some(_), None) => false,
            (None, _) => true,
        };
    if manual_active {
        return Computed { last_date, last_km, next_date: s.manual_due_date, next_km: s.manual_due_km };
    }

    let next_date = match (s.anchor_month, s.anchor_day) {
        (Some(m), Some(d)) => match last_date {
            Some(ld) => Some(next_anchor(ld, m as u32, d as u32)),
            // Never done in the app: keep the deadline already computed so that a missed
            // one turns overdue instead of silently moving to next year.
            None => s.next_due_date.or_else(|| {
                Some(next_anchor(today - chrono::Duration::days(ANCHOR_GRACE_DAYS + 1), m as u32, d as u32))
            }),
        },
        _ => match (s.interval_months, last_date) {
            (Some(m), Some(ld)) => Some(add_months(ld, m)),
            _ => None,
        },
    };
    let next_km = match (s.interval_km, last_km) {
        (Some(i), Some(lk)) => Some(lk + i),
        _ => None,
    };
    Computed { last_date, last_km, next_date, next_km }
}

/// Recomputes and stores the cached due fields of a schedule.
pub async fn recompute(db: &SqlitePool, schedule_id: i64) -> sqlx::Result<()> {
    let Some(s) = sqlx::query_as::<_, Schedule>("SELECT * FROM schedules WHERE id = ?")
        .bind(schedule_id)
        .fetch_optional(db)
        .await?
    else {
        return Ok(());
    };
    let last_event: Option<(NaiveDate, Option<i64>)> = sqlx::query_as(
        "SELECT date, km FROM events WHERE schedule_id = ? ORDER BY date DESC, COALESCE(km, 0) DESC, id DESC LIMIT 1",
    )
    .bind(schedule_id)
    .fetch_optional(db)
    .await?;
    let c = compute(&s, last_event, today());
    sqlx::query(
        "UPDATE schedules SET last_done_date = ?, last_done_km = ?, next_due_date = ?, next_due_km = ? WHERE id = ?",
    )
    .bind(c.last_date)
    .bind(c.last_km)
    .bind(c.next_date)
    .bind(c.next_km)
    .bind(schedule_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn recompute_all(db: &SqlitePool) -> sqlx::Result<()> {
    let ids: Vec<(i64,)> = sqlx::query_as("SELECT id FROM schedules").fetch_all(db).await?;
    for (id,) in ids {
        recompute(db, id).await?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Overdue,
    Soon,
    Setup,
    Ok,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScheduleView {
    #[serde(flatten)]
    pub schedule: Schedule,
    pub vehicle_name: String,
    pub vehicle_color: String,
    pub current_km: Option<i64>,
    pub current_km_date: Option<NaiveDate>,
    pub status: Status,
    pub days_left: Option<i64>,
    pub km_left: Option<i64>,
    pub km_stale: bool,
}

impl ScheduleView {
    pub fn sort_key(&self) -> (Status, i64, i64) {
        (self.status, self.days_left.unwrap_or(i64::MAX / 4), self.km_left.unwrap_or(i64::MAX / 4))
    }
}

pub fn status_of(
    s: &Schedule,
    current_km: Option<i64>,
    current_km_date: Option<NaiveDate>,
    today: NaiveDate,
) -> (Status, Option<i64>, Option<i64>, bool) {
    let days_left = s.next_due_date.map(|d| (d - today).num_days());
    let km_left = match (s.next_due_km, current_km) {
        (Some(n), Some(c)) => Some(n - c),
        _ => None,
    };
    let km_stale = s.next_due_km.is_some()
        && current_km_date.map_or(true, |d| (today - d).num_days() > KM_STALE_DAYS);

    let status = if s.next_due_date.is_none() && s.next_due_km.is_none() {
        Status::Setup
    } else if days_left.is_some_and(|d| d < 0) || km_left.is_some_and(|k| k <= 0) {
        Status::Overdue
    } else if days_left.is_some_and(|d| d <= s.warn_days) || km_left.is_some_and(|k| k <= s.warn_km) {
        Status::Soon
    } else {
        Status::Ok
    };
    (status, days_left, km_left, km_stale)
}

#[derive(sqlx::FromRow)]
struct VehicleKm {
    id: i64,
    name: String,
    color: String,
    km: Option<i64>,
    km_date: Option<NaiveDate>,
}

/// Latest odometer reading per vehicle (highest date, then highest km).
pub const CURRENT_KM_SQL: &str = "
    SELECT v.id, v.name, v.color,
        (SELECT km FROM odometer_readings r WHERE r.vehicle_id = v.id ORDER BY date DESC, km DESC LIMIT 1) AS km,
        (SELECT date FROM odometer_readings r WHERE r.vehicle_id = v.id ORDER BY date DESC, km DESC LIMIT 1) AS km_date
    FROM vehicles v";

pub async fn current_km(db: &SqlitePool, vehicle_id: i64) -> sqlx::Result<(Option<i64>, Option<NaiveDate>)> {
    let row: Option<(i64, NaiveDate)> = sqlx::query_as(
        "SELECT km, date FROM odometer_readings WHERE vehicle_id = ? ORDER BY date DESC, km DESC LIMIT 1",
    )
    .bind(vehicle_id)
    .fetch_optional(db)
    .await?;
    Ok(row.map_or((None, None), |(k, d)| (Some(k), Some(d))))
}

/// Loads active schedules with their status, optionally for one vehicle.
/// Archived vehicles are excluded unless a vehicle is requested explicitly.
pub async fn load_views(db: &SqlitePool, vehicle_id: Option<i64>, include_inactive: bool) -> sqlx::Result<Vec<ScheduleView>> {
    let vehicles: Vec<VehicleKm> = match vehicle_id {
        Some(id) => sqlx::query_as(&format!("{CURRENT_KM_SQL} WHERE v.id = ?")).bind(id).fetch_all(db).await?,
        None => sqlx::query_as(&format!("{CURRENT_KM_SQL} WHERE v.archived = 0")).fetch_all(db).await?,
    };
    let schedules: Vec<Schedule> = sqlx::query_as("SELECT * FROM schedules ORDER BY id").fetch_all(db).await?;
    let today = today();
    let mut views = Vec::new();
    for s in schedules {
        if !include_inactive && !s.active {
            continue;
        }
        let Some(v) = vehicles.iter().find(|v| v.id == s.vehicle_id) else { continue };
        let (status, days_left, km_left, km_stale) = status_of(&s, v.km, v.km_date, today);
        views.push(ScheduleView {
            schedule: s,
            vehicle_name: v.name.clone(),
            vehicle_color: v.color.clone(),
            current_km: v.km,
            current_km_date: v.km_date,
            status,
            days_left,
            km_left,
            km_stale,
        });
    }
    views.sort_by_key(|v| v.sort_key());
    Ok(views)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn sched() -> Schedule {
        Schedule {
            id: 1,
            vehicle_id: 1,
            task_type_id: None,
            title: "Tagliando".into(),
            category: "manutenzione".into(),
            icon: "wrench".into(),
            interval_months: Some(12),
            interval_km: Some(15000),
            anchor_month: None,
            anchor_day: None,
            baseline_date: None,
            baseline_km: None,
            manual_due_date: None,
            manual_due_km: None,
            manual_set_on: None,
            last_done_date: None,
            last_done_km: None,
            next_due_date: None,
            next_due_km: None,
            warn_days: 30,
            warn_km: 2000,
            email_enabled: true,
            active: true,
            notes: None,
        }
    }

    #[test]
    fn from_baseline() {
        let mut s = sched();
        s.baseline_date = Some(d("2026-03-10"));
        s.baseline_km = Some(15000);
        let c = compute(&s, None, d("2026-10-26"));
        assert_eq!(c.next_date, Some(d("2027-03-10")));
        assert_eq!(c.next_km, Some(30000));
    }

    #[test]
    fn event_overrides_baseline_and_manual() {
        let mut s = sched();
        s.baseline_date = Some(d("2025-01-01"));
        s.manual_due_date = Some(d("2026-12-01"));
        s.manual_set_on = Some(d("2026-01-01"));
        let c = compute(&s, Some((d("2026-05-01"), Some(20000))), d("2026-10-26"));
        assert_eq!(c.next_date, Some(d("2027-05-01")));
        assert_eq!(c.next_km, Some(35000));
    }

    #[test]
    fn manual_wins_over_older_event() {
        let mut s = sched();
        s.manual_due_date = Some(d("2026-12-01"));
        s.manual_set_on = Some(d("2026-06-01"));
        let c = compute(&s, Some((d("2026-01-01"), None)), d("2026-10-26"));
        assert_eq!(c.next_date, Some(d("2026-12-01")));
    }

    #[test]
    fn anchored_yearly() {
        let mut s = sched();
        s.interval_months = None;
        s.interval_km = None;
        s.anchor_month = Some(11);
        s.anchor_day = Some(15);
        assert_eq!(compute(&s, None, d("2026-10-26")).next_date, Some(d("2026-11-15")));
        assert_eq!(compute(&s, Some((d("2026-10-30"), None)), d("2026-10-31")).next_date, Some(d("2027-11-15")));
        assert_eq!(compute(&s, None, d("2026-12-01")).next_date, Some(d("2027-11-15")));
    }

    #[test]
    fn statuses() {
        let today = d("2026-10-26");
        let mut s = sched();
        s.next_due_km = Some(30000);
        assert_eq!(status_of(&s, Some(30420), Some(today), today).0, Status::Overdue);
        assert_eq!(status_of(&s, Some(28500), Some(today), today).0, Status::Soon);
        assert_eq!(status_of(&s, Some(20000), Some(today), today).0, Status::Ok);
        assert!(status_of(&s, Some(20000), Some(d("2026-08-01")), today).3);
        s.next_due_km = None;
        assert_eq!(status_of(&s, None, None, today).0, Status::Setup);
        s.next_due_date = Some(d("2026-10-31"));
        assert_eq!(status_of(&s, None, None, today).0, Status::Soon);
        s.next_due_date = Some(d("2026-10-25"));
        assert_eq!(status_of(&s, None, None, today).0, Status::Overdue);
    }
}
