use std::sync::atomic::{AtomicBool, Ordering};

static MINIMIZE_TO_TRAY: AtomicBool = AtomicBool::new(true);
static CLOSE_TO_TRAY: AtomicBool = AtomicBool::new(false);

pub fn set_minimize_to_tray(minimize_to_tray: bool) {
    MINIMIZE_TO_TRAY.store(minimize_to_tray, Ordering::Relaxed);
}

pub fn set_close_to_tray(close_to_tray: bool) {
    CLOSE_TO_TRAY.store(close_to_tray, Ordering::Relaxed);
}

pub fn minimize_to_tray_enabled() -> bool {
    MINIMIZE_TO_TRAY.load(Ordering::Relaxed)
}

pub fn close_to_tray_enabled() -> bool {
    CLOSE_TO_TRAY.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_tray_settings() {
        set_minimize_to_tray(false);
        set_close_to_tray(true);

        assert!(!minimize_to_tray_enabled());
        assert!(close_to_tray_enabled());

        set_minimize_to_tray(true);
        set_close_to_tray(false);

        assert!(minimize_to_tray_enabled());
        assert!(!close_to_tray_enabled());
    }
}
