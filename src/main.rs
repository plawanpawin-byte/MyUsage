#![windows_subsystem = "windows"]

mod app;
mod autostart;
mod cli;
mod config;
mod icon;
mod provider;
mod single_instance;
mod tray;

use std::time::Duration;

use provider::local_cli::LocalCliProvider;
use provider::mock::MockProvider;
use provider::UsageProvider;

const ACCENT_CLAUDE: [u8; 3] = [0xd9, 0x77, 0x57];
const ACCENT_PLUS: [u8; 3] = [0x4f, 0x8c, 0xff];

fn build_providers() -> Vec<Box<dyn UsageProvider>> {
    let home = dirs::home_dir().unwrap_or_default();

    let claude = LocalCliProvider::new(
        "claude_pro",
        "Claude Pro",
        ACCENT_CLAUDE,
        home.join(".claude"),
        MockProvider::new(
            "claude_pro",
            "Claude Pro",
            ACCENT_CLAUDE,
            chrono::Duration::hours(5),
            18.0,
            0.25,
        ),
    );

    let plus = LocalCliProvider::new(
        "codex_plus",
        "Plus",
        ACCENT_PLUS,
        home.join(".codex"),
        MockProvider::new(
            "codex_plus",
            "Plus",
            ACCENT_PLUS,
            chrono::Duration::days(7),
            32.0,
            0.05,
        ),
    );

    vec![Box::new(claude), Box::new(plus)]
}

fn main() -> eframe::Result<()> {
    match cli::parse_args() {
        cli::Mode::Help => {
            attach_parent_console();
            cli::print_help();
            Ok(())
        }
        cli::Mode::Version => {
            attach_parent_console();
            cli::print_version();
            Ok(())
        }
        cli::Mode::Status => {
            attach_parent_console();
            cli::print_status(build_providers());
            Ok(())
        }
        cli::Mode::Gui { start_hidden } => run_gui(start_hidden),
    }
}

/// The binary is built with `windows_subsystem = "windows"` so launching the
/// widget never flashes a console window. That also means stdout/stderr
/// aren't wired to the caller's terminal by default, so text-output modes
/// (`--status`, `--help`, `--version`) would otherwise print into the void.
/// Attaching to the parent console before printing fixes that.
fn attach_parent_console() {
    #[cfg(windows)]
    unsafe {
        windows_sys::Win32::System::Console::AttachConsole(
            windows_sys::Win32::System::Console::ATTACH_PARENT_PROCESS,
        );
    }
}

fn run_gui(start_hidden: bool) -> eframe::Result<()> {
    if !single_instance::acquire() {
        eprintln!("MyUsage กำลังทำงานอยู่แล้ว — ดูที่ System Tray มุมขวาล่าง");
        return Ok(());
    }

    let cfg = config::load();
    let icon_rgba = icon::generate_rgba(64, ACCENT_PLUS);

    let tray_handle = tray::build(icon::generate_rgba(32, ACCENT_PLUS), 32)
        .expect("ไม่สามารถสร้าง System Tray icon ได้");

    let viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([300.0, 320.0])
        .with_min_inner_size([260.0, 200.0])
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top()
        .with_resizable(true)
        .with_visible(!start_hidden)
        .with_icon(eframe::egui::IconData {
            rgba: icon_rgba,
            width: 64,
            height: 64,
        });

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "MyUsage",
        options,
        Box::new(move |cc| {
            let providers = build_providers();
            let refresh_interval = Duration::from_secs(cfg.refresh_interval_secs);
            Ok(Box::new(app::UsageApp::new(
                cc,
                providers,
                refresh_interval,
                tray_handle,
            )))
        }),
    )
}
