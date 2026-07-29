use crate::provider::UsageProvider;

pub enum Mode {
    Gui { start_hidden: bool },
    Status,
    Help,
    Version,
}

pub fn parse_args() -> Mode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--status") | Some("-s") | Some("status") => Mode::Status,
        Some("--help") | Some("-h") => Mode::Help,
        Some("--version") | Some("-v") => Mode::Version,
        Some("--tray") | Some("--hidden") => Mode::Gui { start_hidden: true },
        _ => Mode::Gui { start_hidden: false },
    }
}

pub fn print_help() {
    println!(
        r#"MyUsage - AI Usage Monitor สำหรับ Windows

การใช้งาน:
  MyUsage                  เปิดวิดเจ็ตแสดงการใช้งาน AI
  MyUsage --tray            เปิดแบบซ่อนไปที่ System Tray ทันที (ใช้ตอน Start with Windows)
  MyUsage --status, -s      พิมพ์สถานะการใช้งานปัจจุบันออกทาง terminal แล้วออก
  MyUsage --version, -v     แสดงเวอร์ชัน
  MyUsage --help, -h        แสดงข้อความนี้

คลิกขวาที่ไอคอนใน System Tray เพื่อเปิดหน้าต่าง / ตั้งค่าเริ่มอัตโนมัติเมื่อเปิดเครื่อง / ออกจากโปรแกรม"#
    );
}

pub fn print_version() {
    println!("MyUsage v{}", env!("CARGO_PKG_VERSION"));
}

pub fn print_status(mut providers: Vec<Box<dyn UsageProvider>>) {
    println!("MyUsage - AI Usage Status");
    println!("──────────────────────────────────────────────");
    for provider in providers.iter_mut() {
        let snap = provider.fetch();
        let left = snap.percent_left();
        let reset = snap
            .reset_at
            .map(|r| r.format("%H:%M:%S %d/%m/%Y").to_string())
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<14} {:>5.0}% LEFT   reset: {}",
            snap.display_name, left, reset
        );
        if let Some(note) = snap.note {
            println!("   * {note}");
        }
    }
    println!("──────────────────────────────────────────────");
}
