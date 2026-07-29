use std::time::{Duration, Instant};

use eframe::egui;

use crate::provider::{UsageProvider, UsageSnapshot};
use crate::{autostart, tray};

pub struct UsageApp {
    providers: Vec<Box<dyn UsageProvider>>,
    snapshots: Vec<UsageSnapshot>,
    last_refresh: Instant,
    refresh_interval: Duration,
    tray: tray::TrayHandle,
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
}

impl eframe::App for UsageApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::from_rgba_unmultiplied(0.0, 0.0, 0.0, 0.0).to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_tray_events(ctx);

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
            .fill(egui::Color32::from_rgb(0x12, 0x12, 0x16))
            .rounding(14.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x2a, 0x2a, 0x33)))
            .inner_margin(egui::Margin::same(10.0));

        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            draw_title_bar(ui, ctx);
            ui.add_space(4.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for snap in &self.snapshots {
                    draw_card(ui, snap);
                    ui.add_space(8.0);
                }
            });
        });

        // Keep the countdown / tray-event polling ticking even with no input.
        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

fn dark_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.window_rounding = egui::Rounding::same(14.0);
    visuals.panel_fill = egui::Color32::from_rgb(0x12, 0x12, 0x16);
    visuals
}

fn draw_title_bar(ui: &mut egui::Ui, ctx: &egui::Context) {
    let response = ui
        .horizontal(|ui| {
            ui.label(
                egui::RichText::new("MyUsage")
                    .color(egui::Color32::WHITE)
                    .strong()
                    .size(14.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(egui::Button::new(egui::RichText::new("×").size(16.0)).frame(false))
                    .on_hover_text("ซ่อนไปที่ System Tray")
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                }
            });
        })
        .response;

    // Drag anywhere on the title bar to move the borderless window.
    let drag_area = ui.interact(
        response.rect,
        ui.id().with("title_drag"),
        egui::Sense::click_and_drag(),
    );
    if drag_area.dragged() {
        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
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

fn draw_card(ui: &mut egui::Ui, snap: &UsageSnapshot) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(0x1c, 0x1c, 0x24))
        .rounding(12.0)
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
                let painter = ui.painter();
                painter.circle_filled(
                    rect.center(),
                    15.0,
                    egui::Color32::from_rgb(snap.color[0], snap.color[1], snap.color[2]),
                );
                let initial = snap.display_name.chars().next().unwrap_or('?').to_string();
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    initial,
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                );

                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(&snap.display_name)
                            .color(egui::Color32::from_rgb(0xcf, 0xcf, 0xd6))
                            .size(13.0),
                    );
                    let left = snap.percent_left();
                    ui.label(
                        egui::RichText::new(format!("{left:.0}% LEFT"))
                            .color(left_color(left))
                            .strong()
                            .size(17.0),
                    );
                });
            });

            ui.add_space(6.0);
            let left_frac = snap.percent_left() / 100.0;
            ui.add(
                egui::ProgressBar::new(left_frac)
                    .fill(left_color(snap.percent_left()))
                    .desired_height(6.0)
                    .rounding(3.0),
            );

            if let Some(reset_at) = snap.reset_at {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("Reset Available Remaining: {}", fmt_countdown(reset_at)))
                        .color(egui::Color32::from_rgb(0x9a, 0x9a, 0xa5))
                        .size(11.0),
                );
                ui.label(
                    egui::RichText::new(reset_at.format("คืนโควตา %H:%M:%S %d/%m/%Y").to_string())
                        .color(egui::Color32::from_rgb(0x60, 0x60, 0x6a))
                        .size(10.0),
                );
            }

            if let Some(note) = &snap.note {
                ui.add_space(3.0);
                ui.label(
                    egui::RichText::new(note)
                        .color(egui::Color32::from_rgb(0xb0, 0x8c, 0x3a))
                        .italics()
                        .size(9.5),
                );
            }
        });
}
