//! Appends timestamped diagnostics to `%APPDATA%\MyUsage\crash.log`.
//!
//! The GUI subsystem has no console attached, so a panic or an unexplained
//! visibility change would otherwise leave no trace at all beyond the
//! window vanishing — this gives something concrete to check instead of
//! guessing.
use std::io::Write;

pub fn install() {
    std::panic::set_hook(Box::new(|info| {
        log_event(&format!("panic: {info}"));
    }));
}

pub fn log_event(msg: &str) {
    let Some(dir) = dirs::config_dir().map(|d| d.join("MyUsage")) else {
        return;
    };
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let line = format!("[{}] {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), msg);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("crash.log"))
    {
        let _ = f.write_all(line.as_bytes());
    }
}
