use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub fn register_default_hotkey(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyK);
    let handle = app.clone();

    app.global_shortcut().on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
        // Only toggle on key press, not release
        if event.state != ShortcutState::Pressed {
            return;
        }
        if let Some(window) = handle.get_webview_window("search") {
            let visible = window.is_visible().unwrap_or(false);
            if visible {
                let _ = window.hide();
            } else {
                crate::show_search_window(&handle);
            }
        }
    })?;

    Ok(())
}
