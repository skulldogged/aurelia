//! Appearance-related Tauri command handlers
//!
//! This module contains handlers for appearance settings like window blur modes.

use std::sync::Mutex;
use tauri::{Window, command};

#[cfg(not(target_os = "linux"))]
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
/// On Linux, blur effects are not supported
/// On Windows/macOS, this applies the appropriate vibrancy effect
#[tauri::command]
#[specta::specta]
pub fn set_blur_mode(_window: Window, mode: String) -> Result<(), String> {
    // On Linux, blur/vibrancy effects are not supported, so we only track the state
    #[cfg(target_os = "linux")]
    {
        let mut current_mode_guard = CURRENT_BLUR_MODE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current_mode_guard = Some(mode.clone());
        return Ok(());
    }

    // Windows and macOS blur handling
    #[cfg(not(target_os = "linux"))]
    {
        let result = match mode.as_str() {
            "none" => {
                // Get the current blur mode to know what to clear
                let current_mode = CURRENT_BLUR_MODE
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());

                // Clear the specific blur type that was previously applied
                if let Some(prev_mode) = current_mode.as_ref() {
                    match prev_mode.as_str() {
                        "acrylic" => clear_acrylic(&_window).map_err(|e| e.to_string())?,
                        "mica" => clear_mica(&_window).map_err(|e| e.to_string())?,
                        "tabbed" => clear_tabbed(&_window).map_err(|e| e.to_string())?,
                        _ => clear_blur(&_window).map_err(|e| e.to_string())?,
                    }
                } else {
                    clear_blur(&_window).map_err(|e| e.to_string())?;
                }

                Ok(())
            }
            "acrylic" => apply_acrylic(&_window, None).map_err(|e| e.to_string()),
            "mica" => apply_mica(&_window, None).map_err(|e| e.to_string()),
            "tabbed" => apply_tabbed(&_window, None).map_err(|e| e.to_string()),
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
}
