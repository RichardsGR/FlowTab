use crate::browsers::{self, Browser, BrowserType, OpenTab};
use crate::db::queries;
use crate::state::AppState;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn get_open_tabs(
    browser: Option<String>,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<OpenTab>, String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    let browser_key = browser.unwrap_or_else(|| {
        queries::get_setting(&app.db, "active_browser")
            .ok()
            .flatten()
            .unwrap_or_else(|| "chrome".into())
    });
    drop(app);

    let browser_type = match browser_key.as_str() {
        "edge" => BrowserType::Edge,
        "firefox" => BrowserType::Firefox,
        _ => BrowserType::Chrome,
    };

    let browser = browsers::create_browser(&browser_type)?;
    browser.get_open_tabs()
}

#[tauri::command]
pub fn switch_or_open_tab(
    url: String,
    force_new: Option<bool>,
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let force_new = force_new.unwrap_or(false);

    let app = state.lock().map_err(|e| e.to_string())?;
    let browser_key = queries::get_setting(&app.db, "active_browser")
        .ok()
        .flatten()
        .unwrap_or_else(|| "chrome".into());
    drop(app);

    let browser_type = match browser_key.as_str() {
        "edge" => BrowserType::Edge,
        "firefox" => BrowserType::Firefox,
        _ => BrowserType::Chrome,
    };

    let browser = browsers::create_browser(&browser_type)?;

    if force_new {
        browser.open_new_tab(&url)?;
        return Ok("new_tab".into());
    }

    // Check if URL is already open
    let tabs = browser.get_open_tabs()?;

    // Match by exact URL or prefix (for URLs with trailing slashes, query params, etc.)
    let normalized_url = url.trim_end_matches('/').to_lowercase();
    let matching_tab = tabs.iter().find(|t| {
        let tab_url = t.url.trim_end_matches('/').to_lowercase();
        tab_url == normalized_url || tab_url.starts_with(&normalized_url)
    });

    match matching_tab {
        Some(tab) => {
            browser.activate_tab(tab)?;
            Ok("switched".into())
        }
        None => {
            browser.open_new_tab(&url)?;
            Ok("new_tab".into())
        }
    }
}

#[tauri::command]
pub fn open_new_tab(
    url: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    let browser_key = queries::get_setting(&app.db, "active_browser")
        .ok()
        .flatten()
        .unwrap_or_else(|| "chrome".into());
    drop(app);

    let browser_type = match browser_key.as_str() {
        "edge" => BrowserType::Edge,
        "firefox" => BrowserType::Firefox,
        _ => BrowserType::Chrome,
    };

    let browser = browsers::create_browser(&browser_type)?;
    browser.open_new_tab(&url)
}

#[tauri::command]
pub fn close_tab(
    url: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    let browser_key = queries::get_setting(&app.db, "active_browser")
        .ok()
        .flatten()
        .unwrap_or_else(|| "chrome".into());
    drop(app);

    let browser_type = match browser_key.as_str() {
        "edge" => BrowserType::Edge,
        "firefox" => BrowserType::Firefox,
        _ => BrowserType::Chrome,
    };

    let browser = browsers::create_browser(&browser_type)?;
    let tab = OpenTab { url, title: String::new(), window_id: 1, tab_index: 1 };
    browser.close_tab(&tab)
}
