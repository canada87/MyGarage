//! Serves the SvelteKit build embedded in the binary, with SPA fallback to index.html.

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../frontend/build"]
struct Assets;

fn asset(path: &str) -> Option<Response> {
    let file = Assets::get(path)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let cache = if path.starts_with("_app/immutable/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    Some(
        (
            [(header::CONTENT_TYPE, mime.as_ref().to_string()), (header::CACHE_CONTROL, cache.to_string())],
            Body::from(file.data.into_owned()),
        )
            .into_response(),
    )
}

pub async fn serve(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    if let Some(r) = asset(path) {
        return r;
    }
    asset("index.html").unwrap_or_else(|| {
        (StatusCode::NOT_FOUND, "Frontend non incluso: esegui la build di frontend/ prima di quella del backend").into_response()
    })
}
