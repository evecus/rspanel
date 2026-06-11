use anyhow::{Context, Result};
use bcrypt::{hash, DEFAULT_COST};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::Path,
    sync::{Arc, RwLock},
};

pub static MAIN: RwLock<Option<Arc<MainConfig>>> = RwLock::new(None);
pub static APPS: RwLock<Vec<AppItem>> = RwLock::new(vec![]);
pub static SETTINGS: RwLock<Option<Arc<PanelSettings>>> = RwLock::new(None);

// ── Runtime paths (set once by init()) ───────────────────────────
static DATA_DIR_INNER: RwLock<Option<String>> = RwLock::new(None);
static CONFIG_DIR_INNER: RwLock<Option<String>> = RwLock::new(None);
static CONFIG_PATH_INNER: RwLock<Option<String>> = RwLock::new(None);

pub fn data_dir() -> String {
    DATA_DIR_INNER
        .read()
        .unwrap()
        .clone()
        .unwrap_or_else(|| "data".into())
}
pub fn config_dir() -> String {
    CONFIG_DIR_INNER
        .read()
        .unwrap()
        .clone()
        .unwrap_or_else(|| "data/config".into())
}
pub fn config_path() -> String {
    CONFIG_PATH_INNER
        .read()
        .unwrap()
        .clone()
        .unwrap_or_else(|| "data/config/rspanel.yaml".into())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainConfig {
    pub port: u16,
    pub jwt_secret: String,
    #[serde(default)]
    pub public_mode: bool,
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

impl Default for MainConfig {
    fn default() -> Self {
        Self {
            port: 3088,
            jwt_secret: String::new(),
            public_mode: false,
            users: vec![],
            created_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub url_lan: String,
    #[serde(default)]
    pub url_wan: String,
    #[serde(default)]
    pub icon_type: String,
    #[serde(default)]
    pub icon_text: String,
    #[serde(default)]
    pub icon_image: String,
    #[serde(default)]
    pub open_type: String,
    #[serde(default)]
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClockDisplay {
    #[serde(default)]
    pub show_time: bool,
    #[serde(default)]
    pub show_date: bool,
    #[serde(default)]
    pub show_weekday: bool,
    #[serde(default)]
    pub show_lunar: bool,
    #[serde(default)]
    pub show_seconds: bool,
    #[serde(default)]
    pub show_year: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisplayConfig {
    #[serde(default)]
    pub hostname_size: i32,
    #[serde(default)]
    pub clock_size: i32,
    #[serde(default)]
    pub icon_size: i32,
    #[serde(default)]
    pub app_name_size: i32,
    #[serde(default)]
    pub icon_radius: i32,
    #[serde(default)]
    pub icon_gap: i32,
    #[serde(default)]
    pub side_padding: i32,
    #[serde(default)]
    pub font_hostname: String,
    #[serde(default)]
    pub font_clock: String,
    #[serde(default)]
    pub font_appname: String,
    #[serde(default)]
    pub font_ui: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PanelSettings {
    #[serde(default)]
    pub hostname: String,
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub wallpaper: String,
    #[serde(default)]
    pub clock: ClockDisplay,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub hostname_size: i32,
    #[serde(default)]
    pub clock_size: i32,
    #[serde(default)]
    pub icon_size: i32,
    #[serde(default)]
    pub app_name_size: i32,
    #[serde(default)]
    pub icon_radius: i32,
    #[serde(default)]
    pub icon_gap: i32,
    #[serde(default)]
    pub side_padding: i32,
    #[serde(default)]
    pub font_hostname: String,
    #[serde(default)]
    pub font_clock: String,
    #[serde(default)]
    pub font_appname: String,
    #[serde(default)]
    pub font_ui: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop: Option<DisplayConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<DisplayConfig>,
    #[serde(default)]
    pub network_mode: String,
    #[serde(default)]
    pub show_app_name: bool,
}

pub fn init(base: &str) -> Result<()> {
    let base = base.trim_end_matches('/');
    let cfg_dir = format!("{}/config", base);
    let cfg_path = format!("{}/config/rspanel.yaml", base);

    *DATA_DIR_INNER.write().unwrap() = Some(base.to_string());
    *CONFIG_DIR_INNER.write().unwrap() = Some(cfg_dir);
    *CONFIG_PATH_INNER.write().unwrap() = Some(cfg_path);

    fs::create_dir_all(data_dir())?;
    fs::create_dir_all(format!("{}/uploads", data_dir()))?;
    fs::create_dir_all(config_dir())?;
    load_main()?;
    load_apps()?;
    load_settings()
}

fn load_main() -> Result<()> {
    if !Path::new(&config_path()).exists() {
        // migrate legacy rspanel.yaml from cwd
        if Path::new("rspanel.yaml").exists() {
            let data = fs::read("rspanel.yaml")?;
            fs::write(config_path(), &data)?;
            fs::rename("rspanel.yaml", "rspanel.yaml.bak").ok();
        } else {
            return create_default_main();
        }
    }
    let data = fs::read_to_string(config_path())?;
    let cfg: MainConfig = serde_yaml::from_str(&data).context("parse rspanel.yaml")?;
    *MAIN.write().unwrap() = Some(Arc::new(cfg));
    Ok(())
}

fn create_default_main() -> Result<()> {
    let mut secret = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret);
    let password_hash = hash("admin", DEFAULT_COST)?;
    let now = chrono::Utc::now().to_rfc3339();
    let cfg = MainConfig {
        port: 3088,
        jwt_secret: hex::encode(secret),
        public_mode: false,
        users: vec![User {
            username: "admin".into(),
            password: password_hash,
            nickname: "Admin".into(),
            is_admin: true,
        }],
        created_at: Some(now),
    };
    save_main_inner(&cfg)?;
    *MAIN.write().unwrap() = Some(Arc::new(cfg));
    Ok(())
}

fn save_main_inner(cfg: &MainConfig) -> Result<()> {
    fs::create_dir_all(config_dir())?;
    fs::write(config_path(), serde_yaml::to_string(cfg)?)?;
    Ok(())
}

#[allow(dead_code)]
pub fn save_main() -> Result<()> {
    let guard = MAIN.read().unwrap();
    if let Some(cfg) = guard.as_ref() {
        save_main_inner(cfg)
    } else {
        Ok(())
    }
}

pub fn with_main_mut<F>(f: F) -> Result<()>
where
    F: FnOnce(&mut MainConfig),
{
    let existing = {
        let guard = MAIN.read().unwrap();
        guard.as_ref().map(|arc| (**arc).clone())
    };
    if let Some(mut cfg) = existing {
        f(&mut cfg);
        save_main_inner(&cfg)?;
        *MAIN.write().unwrap() = Some(Arc::new(cfg));
    }
    Ok(())
}

fn load_apps() -> Result<()> {
    let path = format!("{}/apps.json", data_dir());
    if !Path::new(&path).exists() {
        save_apps_inner(&[])?;
        return Ok(());
    }
    let apps: Vec<AppItem> = serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or_default();
    *APPS.write().unwrap() = apps;
    Ok(())
}

fn save_apps_inner(apps: &[AppItem]) -> Result<()> {
    fs::write(
        format!("{}/apps.json", data_dir()),
        serde_json::to_string_pretty(apps)?,
    )?;
    Ok(())
}

pub fn save_apps() -> Result<()> {
    let guard = APPS.read().unwrap();
    save_apps_inner(&guard)
}

fn load_settings() -> Result<()> {
    let path = format!("{}/settings.json", data_dir());
    if !Path::new(&path).exists() {
        let defaults = default_settings();
        save_settings_inner(&defaults)?;
        *SETTINGS.write().unwrap() = Some(Arc::new(defaults));
        return Ok(());
    }
    let s: PanelSettings = serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or_default();
    *SETTINGS.write().unwrap() = Some(Arc::new(s));
    Ok(())
}

fn default_settings() -> PanelSettings {
    PanelSettings {
        hostname: "RsPanel".into(),
        wallpaper: "https://images.unsplash.com/photo-1579546929518-9e396f3cc809?w=1920&q=80"
            .into(),
        theme: "purple-pink".into(),
        language: "zh".into(),
        hostname_size: 72,
        clock_size: 24,
        icon_size: 64,
        app_name_size: 14,
        icon_radius: 25,
        icon_gap: 22,
        side_padding: 49,
        font_hostname: "system".into(),
        font_clock: "system".into(),
        font_appname: "system".into(),
        font_ui: "system".into(),
        show_app_name: true,
        desktop: Some(DisplayConfig {
            hostname_size: 72,
            clock_size: 24,
            icon_size: 64,
            app_name_size: 14,
            icon_radius: 25,
            icon_gap: 22,
            side_padding: 49,
            font_hostname: "system".into(),
            font_clock: "system".into(),
            font_appname: "system".into(),
            font_ui: "system".into(),
        }),
        mobile: Some(DisplayConfig {
            hostname_size: 47,
            clock_size: 17,
            icon_size: 53,
            app_name_size: 11,
            icon_radius: 25,
            icon_gap: 13,
            side_padding: 17,
            font_hostname: "system".into(),
            font_clock: "system".into(),
            font_appname: "system".into(),
            font_ui: "system".into(),
        }),
        clock: ClockDisplay {
            show_time: true,
            show_date: true,
            show_weekday: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn save_settings_inner(s: &PanelSettings) -> Result<()> {
    fs::write(
        format!("{}/settings.json", data_dir()),
        serde_json::to_string_pretty(s)?,
    )?;
    Ok(())
}

pub fn save_settings() -> Result<()> {
    let guard = SETTINGS.read().unwrap();
    if let Some(s) = guard.as_ref() {
        save_settings_inner(s)
    } else {
        Ok(())
    }
}

pub fn find_user(username: &str) -> Option<User> {
    let guard = MAIN.read().unwrap();
    guard
        .as_ref()
        .and_then(|cfg| cfg.users.iter().find(|u| u.username == username).cloned())
}

pub fn get_main() -> Arc<MainConfig> {
    MAIN.read().unwrap().as_ref().unwrap().clone()
}
pub fn get_settings() -> Arc<PanelSettings> {
    SETTINGS.read().unwrap().as_ref().unwrap().clone()
}
pub fn get_apps() -> Vec<AppItem> {
    APPS.read().unwrap().clone()
}

pub fn add_app(mut app: AppItem) {
    let mut guard = APPS.write().unwrap();
    app.order = guard.len() as i32;
    guard.push(app);
}

pub fn update_app(id: &str, mut updated: AppItem) -> bool {
    let mut guard = APPS.write().unwrap();
    for item in guard.iter_mut() {
        if item.id == id {
            updated.id = id.to_string();
            updated.order = item.order;
            *item = updated;
            return true;
        }
    }
    false
}

pub fn delete_app(id: &str) -> bool {
    let mut guard = APPS.write().unwrap();
    if let Some(pos) = guard.iter().position(|a| a.id == id) {
        guard.remove(pos);
        true
    } else {
        false
    }
}

pub fn reorder_apps(ids: &[String]) {
    let mut guard = APPS.write().unwrap();
    let mut map: std::collections::HashMap<String, AppItem> =
        guard.drain(..).map(|a| (a.id.clone(), a)).collect();
    for (i, id) in ids.iter().enumerate() {
        if let Some(mut app) = map.remove(id) {
            app.order = i as i32;
            guard.push(app);
        }
    }
}
