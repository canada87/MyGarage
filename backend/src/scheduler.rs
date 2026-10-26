//! Daily job: refreshes deadlines and sends reminder emails.
//!
//! Per deadline (identified by its due date/km) at most one "preavviso" when it becomes due soon,
//! one "scadenza" on the due day and one "sollecito" a week while it stays overdue.

use std::time::Duration;

use chrono::{Local, NaiveDate, Timelike};

use crate::{
    due::{self, ScheduleView, Status},
    ics,
    mailer::{self, html_escape, html_wrap, Mail},
    settings::Settings,
    AppState,
};

const REMINDER_EVERY_DAYS: i64 = 7;

pub async fn run(state: AppState) {
    loop {
        if let Err(e) = tick(&state).await {
            tracing::error!("scheduler: {e:#}");
        }
        tokio::time::sleep(Duration::from_secs(600)).await;
    }
}

async fn tick(state: &AppState) -> anyhow::Result<()> {
    let settings = Settings::load(&state.db).await?;
    let now = Local::now();
    let today = now.date_naive().to_string();
    if settings.last_run_date == today || now.hour() < settings.send_hour {
        return Ok(());
    }
    due::recompute_all(&state.db).await?;
    if settings.email_enabled {
        notify(state, &settings).await?;
    }
    Settings::set(&state.db, "last_run_date", &today).await?;
    Ok(())
}

fn due_ref(v: &ScheduleView) -> String {
    format!(
        "{}|{}",
        v.schedule.next_due_date.map(|d| d.to_string()).unwrap_or_default(),
        v.schedule.next_due_km.map(|k| k.to_string()).unwrap_or_default()
    )
}

async fn last_sent(state: &AppState, schedule_id: i64, kind: &str, due_ref: &str) -> sqlx::Result<Option<NaiveDate>> {
    sqlx::query_scalar(
        "SELECT date(MAX(sent_at)) FROM notification_log WHERE schedule_id = ? AND kind = ? AND due_ref = ?",
    )
    .bind(schedule_id)
    .bind(kind)
    .bind(due_ref)
    .fetch_one(&state.db)
    .await
}

async fn notify(state: &AppState, settings: &Settings) -> anyhow::Result<()> {
    let today = due::today();
    for v in due::load_views(&state.db, None, false).await? {
        if !v.schedule.email_enabled {
            continue;
        }
        let r = due_ref(&v);
        let kind = match v.status {
            Status::Overdue => {
                let sent = last_sent(state, v.schedule.id, "sollecito", &r).await?;
                if sent.is_some_and(|d| (today - d).num_days() < REMINDER_EVERY_DAYS) {
                    continue;
                }
                "sollecito"
            }
            Status::Soon if v.days_left == Some(0) => "scadenza",
            Status::Soon => "preavviso",
            Status::Ok | Status::Setup => continue,
        };
        if kind != "sollecito" && last_sent(state, v.schedule.id, kind, &r).await?.is_some() {
            continue;
        }
        match mailer::send(settings, compose(&v, &state.base_url)).await {
            Ok(()) => {
                sqlx::query("INSERT INTO notification_log (schedule_id, kind, due_ref) VALUES (?, ?, ?)")
                    .bind(v.schedule.id)
                    .bind(kind)
                    .bind(&r)
                    .execute(&state.db)
                    .await?;
                tracing::info!("email '{kind}' sent for schedule {}", v.schedule.id);
            }
            // Stop at the first failure: settings are probably wrong, retry tomorrow.
            Err(e) => return Err(e.context("invio email")),
        }
    }
    Ok(())
}

fn fmt_date(d: NaiveDate) -> String {
    d.format("%d/%m/%Y").to_string()
}

fn fmt_km(k: i64) -> String {
    let s = k.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push('.');
        }
        out.push(c);
    }
    if k < 0 {
        format!("-{out}")
    } else {
        out
    }
}

/// Human description of how close a deadline is, e.g. "scade tra 5 giorni".
pub fn describe(v: &ScheduleView) -> String {
    let mut parts = Vec::new();
    if let (Some(d), Some(days)) = (v.schedule.next_due_date, v.days_left) {
        parts.push(match days {
            d0 if d0 < 0 => format!("scaduta il {} ({} giorni fa)", fmt_date(d), -d0),
            0 => "scade oggi".to_string(),
            1 => "scade domani".to_string(),
            n => format!("scade il {} (tra {n} giorni)", fmt_date(d)),
        });
    }
    if let (Some(k), Some(left)) = (v.schedule.next_due_km, v.km_left) {
        parts.push(if left <= 0 {
            format!("superati i {} km di {} km", fmt_km(k), fmt_km(-left))
        } else {
            format!("mancano {} km ({} km)", fmt_km(left), fmt_km(k))
        });
    }
    parts.join(" · ")
}

fn compose(v: &ScheduleView, base_url: &str) -> Mail {
    let s = &v.schedule;
    let status = describe(v);
    let subject = match v.status {
        Status::Overdue => format!("Scaduta: {} – {}", s.title, v.vehicle_name),
        _ => format!("{} – {}: {}", s.title, v.vehicle_name, status),
    };
    let link = format!("{base_url}/auto/{}", s.vehicle_id);
    let mut text = format!("{} di {}: {}.\n", s.title, v.vehicle_name, status);
    let mut body = format!(
        "<p><strong>{}</strong> di <strong>{}</strong>: {}.</p>",
        html_escape(&s.title),
        html_escape(&v.vehicle_name),
        html_escape(&status)
    );
    if v.km_stale {
        text.push_str("I km dell'auto non vengono aggiornati da un po': aggiornali per un avviso preciso.\n");
        body.push_str("<p>I km dell'auto non vengono aggiornati da un po': aggiornali per un avviso preciso.</p>");
    }
    if s.next_due_date.is_some() {
        body.push_str("<p style=\"color:#64748B;font-size:13px\">In allegato l'evento da aggiungere al calendario.</p>");
    }
    text.push_str(&link);
    Mail {
        subject,
        text,
        html: html_wrap(&format!("{} – {}", s.title, v.vehicle_name), &body, &link, "Apri l'auto"),
        ics: s.next_due_date.map(|d| (format!("scadenza-{}.ics", s.id), ics::event(v, d, base_url))),
    }
}

#[cfg(test)]
mod tests {
    use super::fmt_km;

    #[test]
    fn km_format() {
        assert_eq!(fmt_km(30420), "30.420");
        assert_eq!(fmt_km(120000), "120.000");
        assert_eq!(fmt_km(999), "999");
    }
}
