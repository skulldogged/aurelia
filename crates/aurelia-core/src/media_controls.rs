//! OS media controls via souvlaki (SMTC / MPRIS / MPNowPlayingInfoCenter).

use souvlaki::{
    MediaButton, MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig,
};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};

pub use crate::models::NowPlayingPayload;

#[derive(Debug, Clone)]
pub enum MediaEvent {
    Play,
    Pause,
    Toggle,
    Next,
    Previous,
    Stop,
    SeekDelta(f64),
    SetPosition(f64),
}

pub struct MediaControlsState {
    pub controls: Mutex<Option<MediaControls>>,
    pub cached_cover_path: Mutex<Option<PathBuf>>,
    pending_events: Arc<Mutex<VecDeque<MediaEvent>>>,
}

impl Default for MediaControlsState {
    fn default() -> Self {
        Self {
            controls: Mutex::new(None),
            cached_cover_path: Mutex::new(None),
            pending_events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl MediaControlsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// On Windows a hidden backend window is created when `hwnd` is `None`.
    pub fn init(&self, hwnd: Option<*mut std::ffi::c_void>) -> Result<(), String> {
        info!("Initializing OS media controls");

        let hwnd = hwnd.or_else(backend_hwnd);
        let config = PlatformConfig {
            dbus_name: "aurelia",
            display_name: "Aurelia",
            hwnd,
            app_id: Some("dev.pupbrained.aurelia"),
        };

        let mut controls = MediaControls::new(config).map_err(|e| {
            error!("Failed to create media controls: {:?}", e);
            format!("Failed to create media controls: {:?}", e)
        })?;
        let event_queue = Arc::clone(&self.pending_events);
        controls
            .attach(move |event| {
                let mapped = match event {
                    MediaControlEvent::Play => MediaEvent::Play,
                    MediaControlEvent::Pause => MediaEvent::Pause,
                    MediaControlEvent::Toggle => MediaEvent::Toggle,
                    MediaControlEvent::Next => MediaEvent::Next,
                    MediaControlEvent::Previous => MediaEvent::Previous,
                    MediaControlEvent::Stop => MediaEvent::Stop,
                    MediaControlEvent::Seek(direction) => match direction {
                        souvlaki::SeekDirection::Forward => MediaEvent::SeekDelta(10.0),
                        souvlaki::SeekDirection::Backward => MediaEvent::SeekDelta(-10.0),
                    },
                    MediaControlEvent::SetPosition(pos) => {
                        MediaEvent::SetPosition(pos.0.as_secs_f64())
                    }
                    _ => return,
                };

                if let Ok(mut queue) = event_queue.lock() {
                    queue.push_back(mapped);
                }
            })
            .map_err(|e| {
                error!("Failed to attach default media control handler: {:?}", e);
                format!("Failed to attach default media control handler: {:?}", e)
            })?;

        *self.controls.lock().map_err(|e| e.to_string())? = Some(controls);
        info!("OS media controls initialized successfully");
        Ok(())
    }

    pub fn pop_event(&self) -> Option<MediaEvent> {
        self.pending_events.lock().ok()?.pop_front()
    }

    pub fn update_now_playing(&self, payload: NowPlayingPayload) -> Result<(), String> {
        debug!("Updating Now Playing: {:?}", payload);
        let mut guard = self.controls.lock().map_err(|e| e.to_string())?;
        if let Some(controls) = guard.as_mut() {
            controls
                .set_playback(MediaPlayback::Playing { progress: None })
                .map_err(|e| format!("Failed to set playback status: {:?}", e))?;
            controls
                .set_metadata(MediaMetadata {
                    title: Some(&payload.title),
                    artist: payload.artist.as_deref(),
                    album: payload.album.as_deref(),
                    duration: payload.duration.map(std::time::Duration::from_secs_f64),
                    cover_url: payload.cover_url.as_deref(),
                })
                .map_err(|e| format!("Failed to set metadata: {:?}", e))?;
        }
        Ok(())
    }

    pub fn set_playback_status(
        &self,
        is_playing: bool,
        position_secs: Option<f64>,
    ) -> Result<(), String> {
        let mut guard = self.controls.lock().map_err(|e| e.to_string())?;
        if let Some(controls) = guard.as_mut() {
            let progress = position_secs
                .map(|p| souvlaki::MediaPosition(std::time::Duration::from_secs_f64(p)));
            let playback = if is_playing {
                MediaPlayback::Playing { progress }
            } else {
                MediaPlayback::Paused { progress }
            };
            controls
                .set_playback(playback)
                .map_err(|e| format!("Failed to set playback status: {:?}", e))?;
        }
        Ok(())
    }

    pub fn clear_now_playing(&self) -> Result<(), String> {
        debug!("Clearing Now Playing");
        let mut guard = self.controls.lock().map_err(|e| e.to_string())?;
        if let Some(controls) = guard.as_mut() {
            controls
                .set_playback(MediaPlayback::Stopped)
                .map_err(|e| format!("Failed to clear playback: {:?}", e))?;
        }
        Ok(())
    }

    pub fn set_button_enabled(&self, button: &str, enabled: bool) -> Result<(), String> {
        let media_button = match button {
            "play" => MediaButton::Play,
            "pause" => MediaButton::Pause,
            "stop" => MediaButton::Stop,
            "next" => MediaButton::Next,
            "previous" => MediaButton::Previous,
            "seek" => MediaButton::Seek,
            _ => return Err(format!("Unknown button: {button}")),
        };
        let mut guard = self.controls.lock().map_err(|e| e.to_string())?;
        if let Some(controls) = guard.as_mut() {
            controls
                .set_button_enabled(media_button, enabled)
                .map_err(|e| format!("Failed to set button enabled: {:?}", e))?;
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn backend_hwnd() -> Option<*mut std::ffi::c_void> {
    use std::sync::mpsc;
    use std::time::Duration;

    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("aurelia-smtc".into())
        .spawn(move || unsafe {
            use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
            use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
                TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG, WNDCLASSW,
                WS_OVERLAPPED,
            };

            unsafe extern "system" fn wnd_proc(
                hwnd: HWND,
                msg: u32,
                wparam: WPARAM,
                lparam: LPARAM,
            ) -> LRESULT {
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }

            let class_name: Vec<u16> = "AureliaSmtc\0".encode_utf16().collect();
            let window_name: Vec<u16> = "Aurelia\0".encode_utf16().collect();
            let instance = GetModuleHandleW(std::ptr::null());
            let class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: instance,
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };
            let _ = RegisterClassW(&class);
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                WS_OVERLAPPED,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null(),
            );
            let _ = tx.send(if hwnd.is_null() { None } else { Some(hwnd as usize) });
            if hwnd.is_null() {
                return;
            }

            let mut msg = std::mem::zeroed::<MSG>();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        })
        .ok()?;

    rx.recv_timeout(Duration::from_secs(2))
        .ok()
        .flatten()
        .map(|hwnd| hwnd as *mut std::ffi::c_void)
}

#[cfg(not(target_os = "windows"))]
fn backend_hwnd() -> Option<*mut std::ffi::c_void> {
    None
}
