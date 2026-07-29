use tray_icon::menu::{CheckMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// Owns the tray icon + its context menu for the whole process lifetime.
/// Dropping this removes the icon from the notification area, so it must be
/// kept alive on the `UsageApp` for as long as the app runs.
pub struct TrayHandle {
    _tray_icon: TrayIcon,
    pub show_id: MenuId,
    pub autostart_item: CheckMenuItem,
    pub autostart_id: MenuId,
    pub exit_id: MenuId,
}

pub fn build(icon_rgba: Vec<u8>, icon_size: u32) -> anyhow::Result<TrayHandle> {
    let icon = Icon::from_rgba(icon_rgba, icon_size, icon_size)?;

    let menu = Menu::new();
    let show_item = MenuItem::new("เปิดหน้าต่าง MyUsage", true, None);
    let autostart_item = CheckMenuItem::new(
        "เริ่มอัตโนมัติเมื่อเปิดเครื่อง",
        true,
        crate::autostart::is_enabled(),
        None,
    );
    let exit_item = MenuItem::new("ออกจากโปรแกรม", true, None);

    menu.append(&show_item)?;
    menu.append(&autostart_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&exit_item)?;

    let show_id = show_item.id().clone();
    let autostart_id = autostart_item.id().clone();
    let exit_id = exit_item.id().clone();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("MyUsage - AI Usage Monitor")
        .with_icon(icon)
        .build()?;

    Ok(TrayHandle {
        _tray_icon: tray_icon,
        show_id,
        autostart_item,
        autostart_id,
        exit_id,
    })
}
