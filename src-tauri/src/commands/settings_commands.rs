use crate::db::queries;
use crate::state::AppState;
use std::sync::Mutex;
use tauri::{Manager, State};

#[tauri::command]
pub fn hide_search_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("search") {
        w.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_setting(
    key: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Option<String>, String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    queries::get_setting(&app.db, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(
    app: tauri::AppHandle,
    key: String,
    value: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        queries::set_setting(&app_state.db, &key, &value).map_err(|e| e.to_string())?;
    }

    if key == "language" {
        if let Some(tray) = app.tray_by_id("main") {
            let menu = crate::build_tray_menu(&app, &value).map_err(|e| e.to_string())?;
            tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
