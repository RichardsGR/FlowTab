mod commands;
mod db;
mod search;
mod browsers;
mod hotkey;
mod state;
mod tray_i18n;

use state::AppState;
use crate::db::queries;
use std::sync::Mutex;
use tauri::Emitter;
use tauri::Manager;
use tauri::Runtime;
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder};

pub fn run() {
    let app_state = match AppState::new() {
        Ok(state) => state,
        Err(e) => {
            eprintln!("Failed to initialize app state: {}", e);
            show_error_dialog(&e.to_string());
            return;
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(app_state))
        .setup(|app| {
            let handle = app.handle().clone();

            // Register global hotkey
            if let Err(e) = hotkey::register_default_hotkey(&handle) {
                eprintln!("Failed to register hotkey: {}", e);
            }

            // Configure macOS window for proper rounded corners
            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("search") {
                use tauri::window::{Effect, EffectsBuilder};
                let effects = EffectsBuilder::new()
                    .effect(Effect::HudWindow)
                    .radius(16.0)
                    .build();
                let _ = window.set_effects(Some(effects));
            }

            // Build tray menu in the saved language
            let lang = {
                let state = app.state::<Mutex<AppState>>();
                let stored = match state.lock() {
                    Ok(guard) => queries::get_setting(&guard.db, "language")
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| "en".into()),
                    Err(_) => "en".into(),
                };
                stored
            };

            let menu = build_tray_menu(&handle, &lang)?;

            let tray_icon = tauri::image::Image::from_bytes(
                include_bytes!("../icons/tray-icon.png")
            ).expect("Failed to load tray icon");

            let _tray = TrayIconBuilder::with_id("main")
                .icon(tray_icon)
                .icon_as_template(false)
                .menu(&menu)
                .tooltip("FlowTab")
                .on_menu_event(move |app, event| {
                    let id = event.id().as_ref();
                    match id {
                        "show_search" => {
                            if let Some(w) = app.get_webview_window("search") {
                                let _ = w.show();
                                let _ = w.center();
                                let _ = w.set_focus();
                            }
                        }
                        "manage_pages" => {
                            if let Some(w) = app.get_webview_window("search") {
                                let _ = w.show();
                                let _ = w.center();
                                let _ = w.set_focus();
                            }
                            let _ = app.emit("navigate", "manager");
                        }
                        "import" => {
                            if let Some(w) = app.get_webview_window("search") {
                                let _ = w.show();
                                let _ = w.center();
                                let _ = w.set_focus();
                            }
                            let _ = app.emit("navigate", "import");
                        }
                        "settings" => {
                            if let Some(w) = app.get_webview_window("search") {
                                let _ = w.show();
                                let _ = w.center();
                                let _ = w.set_focus();
                            }
                            let _ = app.emit("navigate", "settings");
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("search") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.center();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(&handle)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::page_commands::search_pages,
            commands::page_commands::add_page,
            commands::page_commands::update_page,
            commands::page_commands::delete_page,
            commands::page_commands::list_pages,
            commands::page_commands::clean_duplicate_pages,
            commands::browser_commands::get_open_tabs,
            commands::browser_commands::switch_or_open_tab,
            commands::browser_commands::open_new_tab,
            commands::browser_commands::close_tab,
            commands::bookmark_commands::import_bookmarks,
            commands::bookmark_commands::analyze_history,
            commands::settings_commands::get_setting,
            commands::settings_commands::set_setting,
            commands::settings_commands::hide_search_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running FlowTab");
}

pub(crate) fn build_tray_menu<R: Runtime>(
    app: &tauri::AppHandle<R>,
    lang: &str,
) -> tauri::Result<Menu<R>> {
    let labels = tray_i18n::labels(lang);

    let show_search =
        MenuItemBuilder::with_id("show_search", labels.show_search).build(app)?;
    let manage_pages =
        MenuItemBuilder::with_id("manage_pages", labels.manage_pages).build(app)?;
    let import =
        MenuItemBuilder::with_id("import", labels.import_bookmarks).build(app)?;
    let settings =
        MenuItemBuilder::with_id("settings", labels.settings).build(app)?;
    let quit = MenuItemBuilder::with_id("quit", labels.quit).build(app)?;

    MenuBuilder::new(app)
        .item(&show_search)
        .item(&manage_pages)
        .item(&import)
        .item(&settings)
        .separator()
        .item(&quit)
        .build()
}

fn show_error_dialog(msg: &str) {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"display dialog "FlowTab failed to start:\n\n{}" buttons {{"OK"}} default button "OK" with icon stop with title "FlowTab Error""#,
            msg.replace('"', "\\\"")
        );
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output();
    }
}
