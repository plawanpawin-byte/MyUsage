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

    let Ok(key) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
    else {
        return false; // Key only exists once a user has touched theme
                       // settings at all — Windows' own out-of-box default
                       // is light, so that's the safer unknown-case guess.
    };

    // `SystemUsesLightTheme` is the taskbar/Start color; it can be absent
    // even when the key exists (e.g. never explicitly toggled), so fall
    // back to the general app theme rather than guessing dark outright.
    key.get_value::<u32, _>("SystemUsesLightTheme")
        .or_else(|_| key.get_value::<u32, _>("AppsUseLightTheme"))
        .map(|light| light == 0)
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub fn is_dark_mode() -> bool {
    true
}
