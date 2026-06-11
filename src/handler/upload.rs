use crate::config;
use axum::{extract::Multipart, http::StatusCode, Json};
use rand::RngCore;
use std::path::Path;

pub async fn upload_image(
    multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    upload_file(multipart, "").await
}
pub async fn upload_wallpaper(
    multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    upload_file(multipart, "wp_").await
}
pub async fn upload_logo(
    multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    upload_file(multipart, "logo_").await
}

async fn upload_file(
    mut multipart: Multipart,
    prefix: &str,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let filename = field.file_name().unwrap_or("file").to_string();
        let ext = Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let ext = if ext.is_empty() {
            String::new()
        } else {
            format!(".{}", ext)
        };
        let mut b = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut b);
        let out_filename = format!("{}{}{}", prefix, hex::encode(b), ext);
        let dst = format!("{}/uploads/{}", config::data_dir(), out_filename);
        let data = field.bytes().await.map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error":"read error"})),
            )
        })?;
        tokio::fs::write(&dst, &data).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error":"save failed"})),
            )
        })?;
        return Ok(Json(
            serde_json::json!({"url": format!("/uploads/{}", out_filename)}),
        ));
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error":"no file"})),
    ))
}
