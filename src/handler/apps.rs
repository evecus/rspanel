use crate::config::{self, AppItem};
use axum::{extract::Path, http::StatusCode, Json};
use rand::RngCore;
use serde::Deserialize;

pub async fn create_app(
    Json(mut app): Json<AppItem>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut b = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut b);
    app.id = hex::encode(b);
    if app.open_type.is_empty() {
        app.open_type = "new_tab".into();
    }
    if app.icon_type.is_empty() {
        app.icon_type = "text".into();
    }
    config::add_app(app.clone());
    config::save_apps().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"save failed"})),
        )
    })?;
    Ok(Json(serde_json::to_value(&app).unwrap()))
}

pub async fn update_app(
    Path(id): Path<String>,
    Json(app): Json<AppItem>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !config::update_app(&id, app) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        ));
    }
    config::save_apps().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"save failed"})),
        )
    })?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn delete_app(
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !config::delete_app(&id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        ));
    }
    config::save_apps().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"save failed"})),
        )
    })?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
pub struct ReorderReq {
    pub ids: Vec<String>,
}

pub async fn reorder_apps(
    Json(req): Json<ReorderReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    config::reorder_apps(&req.ids);
    config::save_apps().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"save failed"})),
        )
    })?;
    Ok(Json(serde_json::json!({"ok": true})))
}
