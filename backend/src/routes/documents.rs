use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::header,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use super::clean;
use crate::{
    error::{bad, ApiResult, AppError},
    models::Document,
    AppState,
};

const KINDS: &[&str] = &["libretto", "fattura", "polizza", "foto", "altro"];

#[derive(Deserialize)]
pub struct ListQuery {
    vehicle_id: Option<i64>,
    event_id: Option<i64>,
}

pub async fn list(State(state): State<AppState>, Query(q): Query<ListQuery>) -> ApiResult<Vec<Document>> {
    let rows = sqlx::query_as(
        "SELECT * FROM documents
         WHERE (?1 IS NULL OR vehicle_id = ?1) AND (?2 IS NULL OR event_id = ?2)
         ORDER BY uploaded_at DESC, id DESC",
    )
    .bind(q.vehicle_id)
    .bind(q.event_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

fn parse_id(s: &str) -> Result<Option<i64>, AppError> {
    let s = s.trim();
    if s.is_empty() || s == "null" {
        return Ok(None);
    }
    s.parse().map(Some).map_err(|_| bad("Id non valido"))
}

pub async fn upload(State(state): State<AppState>, mut mp: Multipart) -> ApiResult<Document> {
    let mut title = None;
    let mut kind = "altro".to_string();
    let mut vehicle_id = None;
    let mut event_id = None;
    let mut file: Option<(String, String, Vec<u8>)> = None;

    while let Some(field) = mp.next_field().await.map_err(|e| bad(e.to_string()))? {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "file" => {
                let original = field.file_name().unwrap_or("documento").to_string();
                let mime = field
                    .content_type()
                    .map(str::to_string)
                    .unwrap_or_else(|| mime_guess::from_path(&original).first_or_octet_stream().to_string());
                let bytes = field.bytes().await.map_err(|e| bad(e.to_string()))?;
                file = Some((original, mime, bytes.to_vec()));
            }
            other => {
                let text = field.text().await.map_err(|e| bad(e.to_string()))?;
                match other {
                    "title" => title = clean(Some(text)),
                    "kind" if KINDS.contains(&text.as_str()) => kind = text,
                    "vehicle_id" => vehicle_id = parse_id(&text)?,
                    "event_id" => event_id = parse_id(&text)?,
                    _ => {}
                }
            }
        }
    }

    let (original, mime, bytes) = file.ok_or_else(|| bad("Nessun file caricato"))?;
    if bytes.is_empty() {
        return Err(bad("Il file è vuoto"));
    }
    if let (None, Some(eid)) = (vehicle_id, event_id) {
        vehicle_id = sqlx::query_scalar("SELECT vehicle_id FROM events WHERE id = ?")
            .bind(eid)
            .fetch_optional(&state.db)
            .await?;
    }

    let ext = std::path::Path::new(&original)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_lowercase()))
        .unwrap_or_default();
    let folder = vehicle_id.map_or_else(|| "generale".to_string(), |v| format!("auto-{v}"));
    let stored_name = format!("{folder}/{}{ext}", uuid::Uuid::new_v4());
    let path = state.docs_dir.join(&stored_name);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&path, &bytes).await?;

    let title = title.unwrap_or_else(|| {
        std::path::Path::new(&original).file_stem().and_then(|s| s.to_str()).unwrap_or("Documento").to_string()
    });
    let doc = sqlx::query_as(
        "INSERT INTO documents (vehicle_id, event_id, title, kind, stored_name, original_name, mime, size)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(vehicle_id)
    .bind(event_id)
    .bind(title)
    .bind(kind)
    .bind(&stored_name)
    .bind(&original)
    .bind(mime)
    .bind(bytes.len() as i64)
    .fetch_one(&state.db)
    .await;
    match doc {
        Ok(doc) => Ok(Json(doc)),
        Err(e) => {
            let _ = tokio::fs::remove_file(&path).await;
            Err(e.into())
        }
    }
}

#[derive(Deserialize)]
pub struct DocumentUpdate {
    title: String,
    kind: String,
    vehicle_id: Option<i64>,
    event_id: Option<i64>,
}

pub async fn update(State(state): State<AppState>, Path(id): Path<i64>, Json(d): Json<DocumentUpdate>) -> ApiResult<Document> {
    if d.title.trim().is_empty() {
        return Err(bad("Il titolo è obbligatorio"));
    }
    if !KINDS.contains(&d.kind.as_str()) {
        return Err(bad("Tipo di documento non valido"));
    }
    let doc = sqlx::query_as("UPDATE documents SET title = ?, kind = ?, vehicle_id = ?, event_id = ? WHERE id = ? RETURNING *")
        .bind(d.title.trim())
        .bind(d.kind)
        .bind(d.vehicle_id)
        .bind(d.event_id)
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    Ok(Json(doc))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    let doc: Document = sqlx::query_as("SELECT * FROM documents WHERE id = ?").bind(id).fetch_one(&state.db).await?;
    sqlx::query("DELETE FROM documents WHERE id = ?").bind(id).execute(&state.db).await?;
    let _ = tokio::fs::remove_file(state.docs_dir.join(&doc.stored_name)).await;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct FileQuery {
    #[serde(default)]
    download: bool,
}

pub async fn file(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(q): Query<FileQuery>,
) -> Result<impl IntoResponse, AppError> {
    let doc: Document = sqlx::query_as("SELECT * FROM documents WHERE id = ?").bind(id).fetch_one(&state.db).await?;
    let bytes = tokio::fs::read(state.docs_dir.join(&doc.stored_name))
        .await
        .map_err(|_| bad("File non trovato sul disco: controlla la cartella documenti"))?;
    let disposition = if q.download { "attachment" } else { "inline" };
    let safe_name: String = doc
        .original_name
        .chars()
        .map(|c| if c.is_ascii_graphic() || c == ' ' { c } else { '_' })
        .filter(|c| *c != '"' && *c != '\\')
        .collect();
    Ok((
        [
            (header::CONTENT_TYPE, doc.mime),
            (header::CONTENT_DISPOSITION, format!("{disposition}; filename=\"{safe_name}\"")),
        ],
        Body::from(bytes),
    ))
}
