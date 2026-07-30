use std::time::{Duration, Instant};

use eframe::egui;

use crate::provider::{UsageProvider, UsageSnapshot};
use crate::{autostart, taskbar, tray};

/// Fixed width of each provider chip and the gap between them, in egui
/// logical points. Used both to size the docked window and to lay out the
/// chips inside it, so the two must stay in sync.
const CHIP_W: f32 = 152.0;
const CHIP_GAP: f32 = 4.0;
const HIDE_BTN_W: f32 = 22.0;
/// Horizontal/vertical inner margin of the outer panel frame — must match
/// `panel_frame`'s `inner_margin` below since the docked window size is
/// computed from content width alone and needs to add this back in.
const PANEL_MARGIN_X: f32 = 6.0;
const PANEL_MARGIN_Y: f32 = 3.0;
/// Small visual gap kept between the widget's right edge and the
/// notification-area (clock/tray icons) it docks in front of.
const NOTIFY_GAP: f32 = 6.0;

pub struct UsageApp {
    providers: Vec<Box<dyn UsageProvider>>,
    snapshots: Vec<UsageSnapshot>,
    last_refresh: Instant,
    refresh_interval: Duration,
    tray: tray::TrayHandle,
    last_pos: Option<egui::Pos2>,
    last_size: Option<egui::Vec2>,
}

impl UsageApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        providers: Vec<Box<dyn UsageProvider>>,
        refresh_interval: Duration,
        tray: tray::TrayHandle,
    ) -> Self {
        cc.egui_ctx.set_visuals(dark_visuals());

        let mut app = Self {
            providers,
            snapshots: Vec::new(),
            last_refresh: Instant::now() - refresh_interval, // force first fetch
            refresh_interval,
            tray,
            last_pos: None,
            last_size: None,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        self.snapshots = self.providers.iter_mut().map(|p| p.fetch()).collect();
        self.last_refresh = Instant::now();
    }

    fn handle_tray_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if event.id == self.tray.show_id {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            } else if event.id == self.tray.autostart_id {
                let enable = !autostart::is_enabled();
                if autostart::set_enabled(enable).is_ok() {
                    self.tray.autostart_item.set_checked(enable);
                }
            } else if event.id == self.tray.exit_id {
                std::process::exit(0);
            }
        }

        // Left/double click on the tray icon itself also brings the widget back.
        while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
            if matches!(
                event,
                tray_icon::TrayIconEvent::Click { .. } | tray_icon::TrayIconEvent::DoubleClick { .. }
            ) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
    }

    /// Re-measures the real Windows taskbar every frame and keeps the
    /// (borderless, always-on-top) window locked exactly over its empty
    /// area, just in front of the clock / notification icons. Re-running
    /// this every frame is what makes the widget follow the taskbar if it
    /// moves, resizes, or changes DPI/monitor.
    fn dock_to_taskbar(&mut self, ctx: &egui::Context) {
        let Some(taskbar_px) = taskbar::taskbar_rect() else {
            return; // Shell_TrayWnd not found — leave the window where it is.
        };

        let ppp = ctx.pixels_per_point().max(0.01);
        let to_pt = |v: i32| v as f32 / ppp;

        let tb_left = to_pt(taskbar_px.left);
        let tb_top = to_pt(taskbar_px.top);
        let tb_right = to_pt(taskbar_px.right);
        let tb_bottom = to_pt(taskbar_px.bottom);
        let tb_w = tb_right - tb_left;
        let tb_h = tb_bottom - tb_top;

        let notify_px = taskbar::notify_area_rect();
        let n = self.snapshots.len().max(1) as f32;
        let content_w = CHIP_W * n + CHIP_GAP * n + HIDE_BTN_W + PANEL_MARGIN_X * 2.0;

        let (pos, size) = if taskbar_px.width() >= taskbar_px.height() {
            // Common case: horizontal taskbar docked to the top or bottom
            // screen edge. Dock the strip flush against its left edge of
            // the notification area, matching the taskbar's full height.
            let notify_left = notify_px.map(|r| to_pt(r.left)).unwrap_or(tb_right);
            let right_edge = (notify_left - NOTIFY_GAP).min(tb_right);
            let left_edge = (right_edge - content_w).max(tb_left);
            let width = (right_edge - left_edge).max(60.0);
            let height = tb_h.max(28.0);
            (egui::pos2(left_edge, tb_top), egui::vec2(width, height))
        } else {
            // Vertical taskbar (docked to the left/right screen edge):
            // stack the same strip above the notification area instead.
            let notify_top = notify_px.map(|r| to_pt(r.top)).unwrap_or(tb_bottom);
            let bottom_edge = (notify_top - NOTIFY_GAP).min(tb_bottom);
            let content_h = CHIP_W.min(48.0) * n + CHIP_GAP * n + HIDE_BTN_W + PANEL_MARGIN_Y * 2.0;
            let top_edge = (bottom_edge - content_h).max(tb_top);
            let height = (bottom_edge - top_edge).max(60.0);
            let width = tb_w.max(60.0);
            (egui::pos2(tb_left, top_edge), egui::vec2(width, height))
        };

        if self.last_pos != Some(pos) {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
            self.last_pos = Some(pos);
        }
        if self.last_size != Some(size) {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            self.last_size = Some(size);
        }
    }
}

impl eframe::App for UsageApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::from_rgba_unmultiplied(0.0, 0.0, 0.0, 0.0).to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_tray_events(ctx);
        self.dock_to_taskbar(ctx);

        // Closing the (decoration-less) window hides it to the tray instead
        // of quitting — the tray "ออกจากโปรแกรม" item is the real exit.
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        if self.last_refresh.elapsed() >= self.refresh_interval {
            self.refresh();
        }

        let panel_frame = egui::Frame::none()
            .fill(egui::Color32::from_rgb(0x18, 0x18, 0x1c))
            .rounding(6.0)
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0x2a, 0x2a, 0x33)))
            .inner_margin(egui::Margin::symmetric(PANEL_MARGIN_X, PANEL_MARGIN_Y));

        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            let avail_h = ui.available_height();
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = CHIP_GAP;
                for snap in &self.snapshots {
                    draw_chip(ui, snap, egui::vec2(CHIP_W, avail_h));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new(egui::RichText::new("×").size(13.0)).frame(false))
                        .on_hover_text("ซ่อนไปที่ System Tray")
                        .clicked()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                    }
                });
            });
        });

        // Keep the countdown / tray-event polling / taskbar re-lock ticking
        // even with no input.
        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

fn dark_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.window_rounding = egui::Rounding::same(6.0);
    visuals.panel_fill = egui::Color32::from_rgb(0x18, 0x18, 0x1c);
    visuals
}

fn left_color(percent_left: f32) -> egui::Color32 {
    if percent_left >= 50.0 {
        egui::Color32::from_rgb(0x3d, 0xdc, 0x84)
    } else if percent_left >= 20.0 {
        egui::Color32::from_rgb(0xf5, 0xc5, 0x42)
    } else {
        egui::Color32::from_rgb(0xff, 0x5c, 0x5c)
    }
}

fn fmt_countdown(reset_at: chrono::DateTime<chrono::Local>) -> String {
    let remaining = reset_at.signed_duration_since(chrono::Local::now());
    if remaining <= chrono::Duration::zero() {
        return "00:00:00".to_string();
    }
    let total_secs = remaining.num_seconds();
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

fn chip_tooltip(snap: &UsageSnapshot) -> String {
    let mut text = format!(
        "{}\n{:.0}% LEFT",
        snap.display_name,
        snap.percent_left()
    );
    if let Some(reset_at) = snap.reset_at {
        text.push_str(&format!(
            "\nReset Available Remaining: {}\n{}",
            fmt_countdown(reset_at),
            reset_at.format("คืนโควตา %H:%M:%S %d/%m/%Y")
        ));
    }
    if let Some(note) = &snap.note {
        text.push_str(&format!("\n{note}"));
    }
    text
}

/// A single compact provider card: a colored dot + short name + percent on
/// one line, a thin progress bar below it. Full detail (countdown, exact
/// reset time, notes) lives in the hover tooltip since the docked strip is
/// only as tall as the taskbar itself.
fn draw_chip(ui: &mut egui::Ui, snap: &UsageSnapshot, size: egui::Vec2) {
    let tooltip = chip_tooltip(snap);
    let response = ui
        .allocate_ui(size, |ui| {
            // Content (name + percent text) can occasionally want more
            // room than the fixed chip width allows — clip instead of
            // letting it bleed into the next chip, which would throw off
            // the whole strip's width math against the taskbar dock.
            ui.set_clip_rect(ui.max_rect());
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(0x22, 0x22, 0x29))
                .rounding(4.0)
                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                .show(ui, |ui| {
                    ui.set_width(size.x - 16.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                            ui.painter().circle_filled(
                                rect.center(),
                                4.0,
                                egui::Color32::from_rgb(snap.color[0], snap.color[1], snap.color[2]),
                            );
                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new(&snap.display_name)
                                    .color(egui::Color32::from_rgb(0xcf, 0xcf, 0xd6))
                                    .size(11.0),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let left = snap.percent_left();
                                ui.label(
                                    egui::RichText::new(format!("{left:.0}%"))
                                        .color(left_color(left))
                                        .strong()
                                        .size(13.0),
                                );
                            });
                        });

                        let left_frac = snap.percent_left() / 100.0;
                        ui.add(
                            egui::ProgressBar::new(left_frac)
                                .fill(left_color(snap.percent_left()))
                                .desired_height(4.0)
                                .rounding(2.0),
                        );
                    });
                });
        })
        .response;
    response.on_hover_text(tooltip);
}
