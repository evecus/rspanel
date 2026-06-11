mod config;
mod handler;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use axum_extra::extract::cookie::CookieJar;
use clap::Parser;
use rust_embed::RustEmbed;
use tower_http::services::ServeDir;
use tracing::info;

// ── CLI ───────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "rspanel", about = "RsPanel server")]
struct Cli {
    /// 数据目录路径（默认 ./data）
    #[arg(short = 'd', long = "data", default_value = "data")]
    data_dir: String,
}

// ── Embedded assets ───────────────────────────────────────────────

#[derive(RustEmbed)]
#[folder = "web/dist"]
struct DistAssets;

#[derive(RustEmbed)]
#[folder = "assets"]
struct EmbeddedAssets;

// ── App state ─────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct AppState {
    pub current_user: Option<String>,
    pub user_agent: Option<String>,
}

// ── Auth middleware ───────────────────────────────────────────────

async fn auth_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    mut req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, (StatusCode, axum::Json<serde_json::Value>)> {
    let token_str = jar.get("token").map(|c| c.value().to_string()).or_else(|| {
        headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .filter(|v| v.starts_with("Bearer "))
            .map(|v| v["Bearer ".len()..].to_string())
    });

    let token_str = token_str.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({"error":"unauthorized"})),
        )
    })?;

    let username = handler::auth::verify_token(&token_str).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({"error":"invalid token"})),
        )
    })?;

    let mut new_state = state.clone();
    new_state.current_user = Some(username);
    req.extensions_mut().insert(Arc::new(new_state));
    Ok(next.run(req).await)
}

// User-agent extractor middleware
async fn ua_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut req: Request<Body>,
    next: Next<Body>,
) -> Response {
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let mut new_state = state.clone();
    new_state.user_agent = ua;
    req.extensions_mut().insert(Arc::new(new_state));
    next.run(req).await
}

// ── SPA / static file handler ─────────────────────────────────────

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if let Some(content) = DistAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            [(
                axum::http::header::CONTENT_TYPE,
                mime.essence_str().to_string(),
            )],
            content.data.into_owned(),
        )
            .into_response();
    }

    // SPA fallback
    if let Some(index) = DistAssets::get("index.html") {
        return (
            [(
                axum::http::header::CONTENT_TYPE,
                "text/html; charset=utf-8".to_string(),
            )],
            index.data.into_owned(),
        )
            .into_response();
    }

    StatusCode::NOT_FOUND.into_response()
}

async fn default_wallpaper() -> impl IntoResponse {
    for file in EmbeddedAssets::iter() {
        if let Some(content) = EmbeddedAssets::get(&file) {
            let mime = mime_guess::from_path(file.as_ref()).first_or_octet_stream();
            return (
                [
                    (
                        axum::http::header::CONTENT_TYPE,
                        mime.essence_str().to_string(),
                    ),
                    (
                        axum::http::header::CACHE_CONTROL,
                        "public, max-age=86400".to_string(),
                    ),
                ],
                content.data.into_owned(),
            )
                .into_response();
        }
    }
    StatusCode::NOT_FOUND.into_response()
}

// ── Main ──────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
            let _ = std::env::set_current_dir(&dir);
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rspanel=info,tower_http=warn".into()),
        )
        .init();

    config::init(&cli.data_dir)?;

    let port = config::get_main().port;

    let state = AppState::default();

    // ── Public routes ─────────────────────────────────────────────
    let public = Router::new()
        .route("/api/login", post(handler::auth::login))
        .route("/api/panel", get(handler::panel::get_panel_info))
        .route("/api/apps", get(handler::panel::get_apps))
        .route("/api/checkauth", get(handler::auth::check_auth))
        .route("/api/fetch-icon", get(handler::panel::fetch_icon))
        .route("/api/version", get(handler::auth::get_version))
        .route("/default-wallpaper", get(default_wallpaper));

    // ── Protected routes ──────────────────────────────────────────
    let protected = Router::new()
        .route("/api/logout", post(handler::auth::logout))
        .route("/api/me", get(handler::auth::get_me))
        .route("/api/me/nickname", put(handler::auth::update_nickname))
        .route("/api/me/password", put(handler::auth::update_password))
        .route("/api/apps", post(handler::apps::create_app))
        .route("/api/apps/:id", put(handler::apps::update_app))
        .route("/api/apps/:id", delete(handler::apps::delete_app))
        .route("/api/apps/reorder", post(handler::apps::reorder_apps))
        .route("/api/upload", post(handler::upload::upload_image))
        .route(
            "/api/upload/wallpaper",
            post(handler::upload::upload_wallpaper),
        )
        .route("/api/upload/logo", post(handler::upload::upload_logo))
        .route("/api/settings", get(handler::settings::get_settings))
        .route("/api/settings", put(handler::settings::update_settings))
        .route("/api/publicmode", get(handler::settings::get_public_mode))
        .route("/api/publicmode", put(handler::settings::set_public_mode))
        .route("/api/users", get(handler::auth::list_users))
        .route("/api/users", post(handler::auth::create_user))
        .route("/api/users/:username", delete(handler::auth::delete_user))
        )
        )
        )
        )
        )
        )
        )
        )
        )
        )
        )
        )
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let uploads_dir = format!("{}/uploads", config::data_dir());
    std::fs::create_dir_all(&uploads_dir).ok();

    let app = Router::new()
        .nest_service("/uploads", ServeDir::new(&uploads_dir))
        .merge(public)
        .merge(protected)
        .layer(middleware::from_fn_with_state(state.clone(), ua_middleware))
        .fallback(static_handler)
        .with_state(state);

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    info!("\n🚀 RsPanel running on http://{}", addr);
    println!("\n🚀 RsPanel running on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
