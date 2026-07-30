//! Locates the real Windows taskbar on screen so the widget window can be
//! docked exactly on top of it (Shell_TrayWnd is the taskbar itself,
//! TrayNotifyWnd is the clock / notification-icon area on its right edge).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

#[cfg(windows)]
mod win {
    use super::Rect;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowExW, FindWindowW, GetWindowRect};

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    fn window_rect(hwnd: HWND) -> Option<Rect> {
        if hwnd.is_null() {
            return None;
        }
        unsafe {
            let mut rect = std::mem::zeroed();
            if GetWindowRect(hwnd, &mut rect) != 0 {
                Some(Rect {
                    left: rect.left,
                    top: rect.top,
                    right: rect.right,
                    bottom: rect.bottom,
                })
            } else {
                None
            }
        }
    }

    fn tray_hwnd() -> HWND {
        let class = wide("Shell_TrayWnd");
        unsafe { FindWindowW(class.as_ptr(), std::ptr::null()) }
    }

    /// Bounding rect of the whole taskbar (usually spans the bottom edge of
    /// the primary monitor, but can be on any edge or monitor).
    pub fn taskbar_rect() -> Option<Rect> {
        window_rect(tray_hwnd())
    }

    /// Bounding rect of the clock / notification-icon area, which sits at
    /// the end of the taskbar the widget must dock in front of.
    pub fn notify_area_rect() -> Option<Rect> {
        let tray = tray_hwnd();
        if tray.is_null() {
            return None;
        }
        let class = wide("TrayNotifyWnd");
        let notify = unsafe { FindWindowExW(tray, std::ptr::null_mut(), class.as_ptr(), std::ptr::null()) };
        window_rect(notify)
    }
}

#[cfg(windows)]
pub use win::{notify_area_rect, taskbar_rect};

#[cfg(not(windows))]
pub fn taskbar_rect() -> Option<Rect> {
    None
}

#[cfg(not(windows))]
pub fn notify_area_rect() -> Option<Rect> {
    None
}
