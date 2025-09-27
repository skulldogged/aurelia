use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Listener, Manager};

#[tauri::command]
#[specta::specta]
pub fn show_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
#[specta::specta]
pub fn hide_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
#[specta::specta]
pub fn quit_application(app: AppHandle) {
    app.cleanup_before_exit();
    std::process::exit(0);
}

pub fn setup_system_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let toggle_visibility =
        tauri::menu::MenuItem::with_id(app, "toggle_visibility", "Show/Hide", true, None::<&str>)?;

    let quit = tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = tauri::menu::MenuBuilder::new(app)
        .items(&[&toggle_visibility, &quit])
        .build()?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "toggle_visibility" => {
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            "quit" => {
                app.cleanup_before_exit();
                std::process::exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

use std::sync::atomic::{AtomicBool, Ordering};

static MINIMIZE_TO_TRAY: AtomicBool = AtomicBool::new(true);
static CLOSE_TO_TRAY: AtomicBool = AtomicBool::new(false);

#[tauri::command]
#[specta::specta]
pub fn set_minimize_to_tray(minimize_to_tray: bool) {
    MINIMIZE_TO_TRAY.store(minimize_to_tray, Ordering::Relaxed);
}

#[tauri::command]
#[specta::specta]
pub fn set_close_to_tray(close_to_tray: bool) {
    CLOSE_TO_TRAY.store(close_to_tray, Ordering::Relaxed);
}

pub fn setup_window_behavior(app: &AppHandle) {
    let app_handle_minimize = app.clone();

    // Listen for minimize events
    let _minimize_id = app.listen("tauri://window-minimize", move |_event| {
        if MINIMIZE_TO_TRAY.load(Ordering::Relaxed)
            && let Some(window) = app_handle_minimize.get_webview_window("main")
        {
            let _ = window.hide();
        }
    });

    // Set up window event handler for close events
    if let Some(window) = app.get_webview_window("main") {
        let window_clone = window.clone();
        let _close_handler = window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if CLOSE_TO_TRAY.load(Ordering::Relaxed) {
                    // Prevent default close behavior and hide to tray instead
                    let _ = window_clone.hide();
                    api.prevent_close();
                }
                // If close to tray is disabled, let the default close behavior happen
            }
        });
    }
}
