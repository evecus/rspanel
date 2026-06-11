use axum::{http::StatusCode, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use bcrypt::verify;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{config, AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
}

fn jwt_secret() -> String {
    config::get_main().jwt_secret.clone()
}

pub fn make_token(username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
        + Duration::from_secs(7 * 24 * 3600))
    .as_secs();
    let claims = Claims {
        sub: username.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
}

pub fn verify_token(token: &str) -> Option<String> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &validation,
    )
    .ok()
    .map(|d| d.claims.sub)
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

pub async fn login(
    jar: CookieJar,
    Json(req): Json<LoginReq>,
) -> Result<(CookieJar, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let user = config::find_user(&req.username).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"invalid credentials"})),
        )
    })?;

    let ok = verify(&req.password, &user.password).unwrap_or(false);
    if !ok {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"invalid credentials"})),
        ));
    }

    let token = make_token(&user.username).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"token error"})),
        )
    })?;

    let mut cookie = Cookie::new("token", token.clone());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_max_age(time::Duration::seconds(7 * 24 * 3600));

    Ok((
        jar.add(cookie),
        Json(serde_json::json!({
            "token": token,
            "username": user.username,
            "nickname": user.nickname,
            "is_admin": user.is_admin,
        })),
    ))
}

pub async fn logout(jar: CookieJar) -> (CookieJar, Json<serde_json::Value>) {
    let mut cookie = Cookie::new("token", "");
    cookie.set_path("/");
    (jar.remove(cookie), Json(serde_json::json!({"ok": true})))
}

pub async fn get_me(
    axum::extract::Extension(state): axum::extract::Extension<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let username = state.current_user.clone().unwrap_or_default();
    let user = config::find_user(&username).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"user not found"})),
        )
    })?;
    Ok(Json(
        serde_json::json!({"username": user.username, "nickname": user.nickname, "is_admin": user.is_admin}),
    ))
}

pub async fn check_auth(jar: CookieJar) -> Json<serde_json::Value> {
    let token_str = jar.get("token").map(|c| c.value().to_string());
    let Some(token) = token_str else {
        return Json(serde_json::json!({"logged_in": false}));
    };
    let Some(username) = verify_token(&token) else {
        return Json(serde_json::json!({"logged_in": false}));
    };
    let Some(user) = config::find_user(&username) else {
        return Json(serde_json::json!({"logged_in": false}));
    };
    Json(
        serde_json::json!({"logged_in": true, "username": user.username, "nickname": user.nickname, "is_admin": user.is_admin}),
    )
}

#[derive(Deserialize)]
pub struct UpdatePasswordReq {
    pub old_password: String,
    pub new_password: String,
}

pub async fn update_password(
    axum::extract::Extension(state): axum::extract::Extension<Arc<AppState>>,
    Json(req): Json<UpdatePasswordReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let username = state.current_user.clone().unwrap_or_default();
    let user = config::find_user(&username).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"user not found"})),
        )
    })?;
    if !verify(&req.old_password, &user.password).unwrap_or(false) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"wrong password"})),
        ));
    }
    let new_hash = bcrypt::hash(&req.new_password, bcrypt::DEFAULT_COST).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"hash error"})),
        )
    })?;
    config::with_main_mut(|cfg| {
        if let Some(u) = cfg.users.iter_mut().find(|u| u.username == username) {
            u.password = new_hash;
        }
    })
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"save failed"})),
        )
    })?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
pub struct UpdateNicknameReq {
    pub nickname: String,
}

pub async fn update_nickname(
    axum::extract::Extension(state): axum::extract::Extension<Arc<AppState>>,
    Json(req): Json<UpdateNicknameReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let username = state.current_user.clone().unwrap_or_default();
    config::with_main_mut(|cfg| {
        if let Some(u) = cfg.users.iter_mut().find(|u| u.username == username) {
            u.nickname = req.nickname.clone();
        }
    })
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"save failed"})),
        )
    })?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn list_users() -> Json<serde_json::Value> {
    let main = config::get_main();
    let users: Vec<_> = main.users.iter()
        .map(|u| serde_json::json!({"username": u.username, "nickname": u.nickname, "is_admin": u.is_admin}))
        .collect();
    Json(serde_json::Value::Array(users))
}

#[derive(Deserialize)]
pub struct CreateUserReq {
    pub username: String,
    pub password: String,
    pub nickname: String,
    pub is_admin: bool,
}

pub async fn create_user(
    Json(req): Json<CreateUserReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if config::find_user(&req.username).is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error":"user exists"})),
        ));
    }
    let hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"hash error"})),
        )
    })?;
    config::with_main_mut(|cfg| {
        cfg.users.push(config::User {
            username: req.username.clone(),
            password: hash,
            nickname: req.nickname.clone(),
            is_admin: req.is_admin,
        });
    })
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"save failed"})),
        )
    })?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn delete_user(
    axum::extract::Extension(state): axum::extract::Extension<Arc<AppState>>,
    axum::extract::Path(target): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let me = state.current_user.clone().unwrap_or_default();
    if target == me {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error":"cannot delete self"})),
        ));
    }
    let mut found = false;
    config::with_main_mut(|cfg| {
        if let Some(pos) = cfg.users.iter().position(|u| u.username == target) {
            cfg.users.remove(pos);
            found = true;
        }
    })
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"save failed"})),
        )
    })?;
    if found {
        Ok(Json(serde_json::json!({"ok": true})))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        ))
    }
}

pub async fn get_version() -> Json<serde_json::Value> {
    let version = std::env::var("APP_VERSION").unwrap_or_else(|_| "dev".into());
    Json(serde_json::json!({"version": version}))
}
