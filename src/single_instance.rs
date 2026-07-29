//! Prevents launching a second copy of the widget by holding a named Win32
//! mutex for the lifetime of the process. The handle is intentionally leaked
//! (never closed) so it stays alive until the OS reclaims it on exit.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows_sys::Win32::System::Threading::CreateMutexW;

/// Returns `true` if this process won the race and is the only instance.
pub fn acquire() -> bool {
    let wide_name: Vec<u16> = OsStr::new("Local\\MyUsage_SingleInstance_Mutex")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let handle = CreateMutexW(std::ptr::null(), 0, wide_name.as_ptr());
        if handle.is_null() {
            // Could not create the mutex at all; fail open rather than
            // blocking the user from ever starting the app.
            return true;
        }
        GetLastError() != ERROR_ALREADY_EXISTS
    }
}
