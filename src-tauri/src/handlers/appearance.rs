//! Appearance-related Tauri command handlers
//!
//! This module contains handlers for appearance settings like window blur modes.

use std::sync::Mutex;
use tauri::{Window, command};
use window_vibrancy::{
    apply_acrylic, apply_mica, apply_tabbed, clear_acrylic, clear_blur, clear_mica, clear_tabbed,
};

// Global state to track current blur mode
static CURRENT_BLUR_MODE: Mutex<Option<String>> = Mutex::new(None);

/// Get the current blur mode
#[command]
#[specta::specta]
pub fn get_blur_mode() -> String {
    let mode = CURRENT_BLUR_MODE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    mode.clone().unwrap_or_else(|| "acrylic".to_string())
}

/// Set the window blur mode
#[tauri::command]
#[specta::specta]
pub fn set_blur_mode(window: Window, mode: String) -> Result<(), String> {
    let result = match mode.as_str() {
        "none" => {
            // Get the current blur mode to know what to clear
            let current_mode = CURRENT_BLUR_MODE
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            // Clear the specific blur type that was previously applied
            if let Some(prev_mode) = current_mode.as_ref() {
                match prev_mode.as_str() {
                    "acrylic" => clear_acrylic(&window).map_err(|e| e.to_string())?,
                    "mica" => clear_mica(&window).map_err(|e| e.to_string())?,
                    "tabbed" => clear_tabbed(&window).map_err(|e| e.to_string())?,
                    _ => clear_blur(&window).map_err(|e| e.to_string())?,
                }
            } else {
                clear_blur(&window).map_err(|e| e.to_string())?;
            }

            Ok(())
        }
        "acrylic" => apply_acrylic(&window, None).map_err(|e| e.to_string()),
        "mica" => apply_mica(&window, None).map_err(|e| e.to_string()),
        "tabbed" => apply_tabbed(&window, None).map_err(|e| e.to_string()),
        _ => Err(format!("Unsupported blur mode: {}", mode)),
    };

    match result {
        Ok(_) => {
            // Update global state to track current blur mode
            let mut current_mode_guard = CURRENT_BLUR_MODE
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *current_mode_guard = Some(mode.clone());
            Ok(())
        }
        Err(e) => Err(e),
    }
}
