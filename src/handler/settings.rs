use crate::config::{self, PanelSettings};
use axum::{http::StatusCode, Json};
use std::sync::Arc;

pub async fn get_settings() -> Json<serde_json::Value> {
    Json(serde_json::to_value(&*config::get_settings()).unwrap())
}

pub async fn update_settings(
    Json(new_settings): Json<PanelSettings>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    *config::SETTINGS.write().unwrap() = Some(Arc::new(new_settings));
    config::save_settings().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"save failed"})),
        )
    })?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn get_public_mode() -> Json<serde_json::Value> {
    Json(serde_json::json!({"public_mode": config::get_main().public_mode}))
}

#[derive(serde::Deserialize)]
pub struct PublicModeReq {
    pub public_mode: bool,
}

pub async fn set_public_mode(
    Json(req): Json<PublicModeReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    config::with_main_mut(|cfg| {
        cfg.public_mode = req.public_mode;
    })
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"save failed"})),
        )
    })?;
    Ok(Json(serde_json::json!({"ok": true})))
}
