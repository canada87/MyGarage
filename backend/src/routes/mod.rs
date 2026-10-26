mod contacts;
mod dashboard;
mod documents;
mod events;
mod odometer;
mod schedules;
mod settings;
mod task_types;
mod tyres;
mod vehicles;

use axum::{
    routing::{get, post, put},
    Router,
};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(dashboard::dashboard))
        .route("/expenses", get(dashboard::expenses))
        .route("/vehicles", get(vehicles::list).post(vehicles::create))
        .route("/vehicles/{id}", get(vehicles::get).put(vehicles::update).delete(vehicles::delete))
        .route("/vehicles/{id}/odometer", get(odometer::list).post(odometer::create))
        .route("/odometer/{id}", put(odometer::update).delete(odometer::delete))
        .route("/vehicles/{id}/tyres", get(tyres::list).post(tyres::create))
        .route("/tyre-sets/{id}", put(tyres::update).delete(tyres::delete))
        .route("/tyre-sets/{id}/mount", post(tyres::mount))
        .route("/tyre-sets/{id}/unmount", post(tyres::unmount))
        .route("/tyre-mounts/{id}", put(tyres::update_mount).delete(tyres::delete_mount))
        .route("/task-types", get(task_types::list).post(task_types::create))
        .route("/task-types/{id}", put(task_types::update).delete(task_types::delete))
        .route("/schedules", get(schedules::list).post(schedules::create))
        .route("/schedules/{id}", get(schedules::get).put(schedules::update).delete(schedules::delete))
        .route("/schedules/{id}/calendar.ics", get(schedules::ics))
        .route("/events", get(events::list).post(events::create))
        .route("/events/{id}", put(events::update).delete(events::delete))
        .route("/documents", get(documents::list).post(documents::upload))
        .route("/documents/{id}", put(documents::update).delete(documents::delete))
        .route("/documents/{id}/file", get(documents::file))
        .route("/contacts", get(contacts::list).post(contacts::create))
        .route("/contacts/{id}", put(contacts::update).delete(contacts::delete))
        .route("/settings", get(settings::get).put(settings::update))
        .route("/settings/test-email", post(settings::test_email))
}

/// Trims strings and turns empty ones into `None`.
pub fn clean(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}
