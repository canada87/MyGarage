//! Minimal iCalendar (RFC 5545) output for deadlines, accepted by Outlook and Google Calendar.

use chrono::{NaiveDate, Utc};

use crate::due::ScheduleView;

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace(';', "\\;").replace(',', "\\,").replace('\n', "\\n")
}

/// Folds content lines longer than 75 octets, without splitting UTF-8 characters.
fn fold(line: &str) -> String {
    let mut out = String::new();
    let mut len = 0;
    for ch in line.chars() {
        let n = ch.len_utf8();
        if len + n > 75 {
            out.push_str("\r\n ");
            len = 1;
        }
        out.push(ch);
        len += n;
    }
    out
}

pub fn event(v: &ScheduleView, date: NaiveDate, base_url: &str) -> String {
    let s = &v.schedule;
    let summary = format!("{} – {}", s.title, v.vehicle_name);
    let mut description = format!("Scadenza {} per {}.", s.title, v.vehicle_name);
    if let Some(km) = s.next_due_km {
        description.push_str(&format!("\nOppure a {km} km, se arrivano prima."));
    }
    description.push_str(&format!("\n{base_url}/auto/{}", s.vehicle_id));

    let lines = [
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        "PRODID:-//MyGarage//Scadenze//IT".to_string(),
        "CALSCALE:GREGORIAN".to_string(),
        "METHOD:PUBLISH".to_string(),
        "BEGIN:VEVENT".to_string(),
        format!("UID:mygarage-schedule-{}-{}@mygarage", s.id, date.format("%Y%m%d")),
        format!("DTSTAMP:{}", Utc::now().format("%Y%m%dT%H%M%SZ")),
        format!("DTSTART;VALUE=DATE:{}", date.format("%Y%m%d")),
        format!("DTEND;VALUE=DATE:{}", (date + chrono::Duration::days(1)).format("%Y%m%d")),
        format!("SUMMARY:{}", escape(&summary)),
        format!("DESCRIPTION:{}", escape(&description)),
        "TRANSP:TRANSPARENT".to_string(),
        "BEGIN:VALARM".to_string(),
        "ACTION:DISPLAY".to_string(),
        format!("DESCRIPTION:{}", escape(&summary)),
        "TRIGGER:-P7D".to_string(),
        "END:VALARM".to_string(),
        "END:VEVENT".to_string(),
        "END:VCALENDAR".to_string(),
    ];
    let mut out = String::new();
    for l in lines {
        out.push_str(&fold(&l));
        out.push_str("\r\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_long_lines() {
        let long = "X".repeat(200);
        assert!(fold(&long).split("\r\n").all(|l| l.len() <= 75));
        assert_eq!(escape("a,b;c\nd"), "a\\,b\\;c\\nd");
    }
}
