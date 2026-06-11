use crate::{
    config::{self, DisplayConfig, PanelSettings},
    AppState,
};
use axum::{extract::Extension, Json};
use std::sync::Arc;

fn is_mobile(ua: &str) -> bool {
    let ua = ua.to_lowercase();
    ["android", "iphone", "ipod", "windows phone", "mobile"]
        .iter()
        .any(|kw| ua.contains(kw))
}

fn resolve_disp(d: &DisplayConfig, s: &PanelSettings) -> serde_json::Value {
    serde_json::json!({
        "hostname_size": if d.hostname_size != 0 { d.hostname_size } else if s.hostname_size != 0 { s.hostname_size } else { 56 },
        "clock_size": if d.clock_size != 0 { d.clock_size } else if s.clock_size != 0 { s.clock_size } else { 16 },
        "icon_size": if d.icon_size != 0 { d.icon_size } else if s.icon_size != 0 { s.icon_size } else { 78 },
        "app_name_size": if d.app_name_size != 0 { d.app_name_size } else if s.app_name_size != 0 { s.app_name_size } else { 12 },
        "icon_radius": if d.icon_radius != 0 { d.icon_radius } else if s.icon_radius != 0 { s.icon_radius } else { 26 },
        "icon_gap": if d.icon_gap != 0 { d.icon_gap } else if s.icon_gap != 0 { s.icon_gap } else { 22 },
        "side_padding": if d.side_padding != 0 { d.side_padding } else if s.side_padding != 0 { s.side_padding } else { 52 },
        "font_hostname": if !d.font_hostname.is_empty() { &d.font_hostname } else if !s.font_hostname.is_empty() { &s.font_hostname } else { "system" },
        "font_clock": if !d.font_clock.is_empty() { &d.font_clock } else if !s.font_clock.is_empty() { &s.font_clock } else { "system" },
        "font_appname": if !d.font_appname.is_empty() { &d.font_appname } else if !s.font_appname.is_empty() { &s.font_appname } else { "system" },
        "font_ui": if !d.font_ui.is_empty() { &d.font_ui } else if !s.font_ui.is_empty() { &s.font_ui } else { "system" },
    })
}

pub async fn get_panel_info(Extension(state): Extension<Arc<AppState>>) -> Json<serde_json::Value> {
    let s = config::get_settings();
    let main = config::get_main();
    let mobile = is_mobile(state.user_agent.as_deref().unwrap_or(""));

    let default_disp = DisplayConfig::default();
    let active_disp = if mobile {
        s.mobile.as_ref().unwrap_or(&default_disp)
    } else {
        s.desktop.as_ref().unwrap_or(&default_disp)
    };
    let d = resolve_disp(active_disp, &s);

    let fallback_disp = DisplayConfig {
        hostname_size: s.hostname_size,
        clock_size: s.clock_size,
        icon_size: s.icon_size,
        app_name_size: s.app_name_size,
        icon_radius: s.icon_radius,
        icon_gap: s.icon_gap,
        side_padding: s.side_padding,
        font_hostname: s.font_hostname.clone(),
        font_clock: s.font_clock.clone(),
        font_appname: s.font_appname.clone(),
        font_ui: s.font_ui.clone(),
    };
    let desktop_disp = s.desktop.clone().unwrap_or_else(|| fallback_disp.clone());
    let mobile_disp = s.mobile.clone().unwrap_or(fallback_disp);

    Json(serde_json::json!({
        "hostname": s.hostname, "logo": s.logo, "wallpaper": s.wallpaper,
        "clock": s.clock, "theme": s.theme, "language": s.language,
        "hostname_size": d["hostname_size"], "clock_size": d["clock_size"],
        "icon_size": d["icon_size"], "app_name_size": d["app_name_size"],
        "icon_radius": d["icon_radius"], "icon_gap": d["icon_gap"],
        "side_padding": d["side_padding"], "font_hostname": d["font_hostname"],
        "font_clock": d["font_clock"], "font_appname": d["font_appname"], "font_ui": d["font_ui"],
        "desktop": desktop_disp, "mobile": mobile_disp, "is_mobile": mobile,
        "public_mode": main.public_mode, "network_mode": s.network_mode,
        "show_app_name": s.show_app_name,
    }))
}

pub async fn get_apps() -> Json<serde_json::Value> {
    Json(serde_json::Value::Array(
        config::get_apps()
            .into_iter()
            .map(|a| serde_json::to_value(a).unwrap())
            .collect(),
    ))
}

pub async fn fetch_icon(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let Some(raw_url) = params.get("url") else {
        return Json(serde_json::json!({"error": "url required"}));
    };
    let raw_url = raw_url.trim();
    let raw_url = if !raw_url.starts_with("http://") && !raw_url.starts_with("https://") {
        format!("http://{}", raw_url)
    } else {
        raw_url.to_string()
    };

    let Ok(parsed) = url::Url::parse(&raw_url) else {
        return Json(serde_json::json!({"error": "invalid url"}));
    };
    let base = format!(
        "{}://{}",
        parsed.scheme(),
        parsed.host_str().unwrap_or_default()
    );
    let scheme = parsed.scheme().to_string();

    let icon_url = if let Ok(resp) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(6))
        .build()
        .unwrap()
        .get(&raw_url)
        .send()
        .await
    {
        if let Ok(body) = resp.text().await {
            parse_favicon_from_html(&body, &base, &scheme)
        } else {
            None
        }
    } else {
        None
    };

    let icon_url = icon_url.unwrap_or_else(|| format!("{}/favicon.ico", base));
    Json(serde_json::json!({"icon": icon_url}))
}

fn parse_favicon_from_html(html: &str, base: &str, scheme: &str) -> Option<String> {
    use regex::Regex;
    let re1 = Regex::new(r#"(?i)<link[^>]+rel=["'](?:shortcut icon|icon|apple-touch-icon)["'][^>]+href=["']([^"']+)["']"#).ok()?;
    let re2 = Regex::new(r#"(?i)<link[^>]+href=["']([^"']+)["'][^>]+rel=["'](?:shortcut icon|icon|apple-touch-icon)["']"#).ok()?;
    let href = re1
        .captures(html)
        .and_then(|m| m.get(1))
        .or_else(|| re2.captures(html).and_then(|m| m.get(1)))
        .map(|m| m.as_str().to_string())?;
    let url = if href.starts_with("//") {
        format!("{}:{}", scheme, href)
    } else if href.starts_with('/') {
        format!("{}{}", base, href)
    } else if !href.starts_with("http") {
        format!("{}/{}", base, href)
    } else {
        href
    };
    Some(url)
}
