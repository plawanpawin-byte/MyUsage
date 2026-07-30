//! Whether the taskbar/Start menu ("system" mode) is currently dark or
//! light, so the docked widget can use a neutral color scheme that blends
//! into the real taskbar behind it instead of a fixed color that may clash
//! with it. This is deliberately `SystemUsesLightTheme`, not
//! `AppsUseLightTheme` — Windows lets those two differ (e.g. dark taskbar
//! with light app windows), and it's the taskbar's own color that matters
//! here.
#[cfg(windows)]
pub fn is_dark_mode() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|key| key.get_value::<u32, _>("SystemUsesLightTheme"))
        .map(|light| light == 0)
        .unwrap_or(true)
}

#[cfg(not(windows))]
pub fn is_dark_mode() -> bool {
    true
}
