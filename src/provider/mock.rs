use std::time::Instant;

use chrono::{DateTime, Local};

use super::{UsageProvider, UsageSnapshot};

/// Deterministic, self-contained provider used whenever no real account
/// session could be found (see `local_cli.rs`). It slowly drifts the "used"
/// percentage upward over time and resets on a fixed window, purely so the
/// UI has something live to render out of the box.
pub struct MockProvider {
    id: &'static str,
    display_name: String,
    color: [u8; 3],
    started_at: Instant,
    window: chrono::Duration,
    reset_at: DateTime<Local>,
    starting_percent: f32,
    drift_per_minute: f32,
}

impl MockProvider {
    pub fn new(
        id: &'static str,
        display_name: &str,
        color: [u8; 3],
        window: chrono::Duration,
        starting_percent: f32,
        drift_per_minute: f32,
    ) -> Self {
        Self {
            id,
            display_name: display_name.to_string(),
            color,
            started_at: Instant::now(),
            window,
            reset_at: Local::now() + window,
            starting_percent,
            drift_per_minute,
        }
    }
}

impl UsageProvider for MockProvider {
    fn fetch(&mut self) -> UsageSnapshot {
        let now = Local::now();
        if now >= self.reset_at {
            // Simulate the quota window rolling over.
            self.reset_at = now + self.window;
            self.started_at = Instant::now();
        }

        let elapsed_minutes = self.started_at.elapsed().as_secs_f32() / 60.0;
        let percent_used =
            (self.starting_percent + elapsed_minutes * self.drift_per_minute).clamp(0.0, 100.0);

        UsageSnapshot {
            id: self.id,
            display_name: self.display_name.clone(),
            color: self.color,
            percent_used,
            reset_at: Some(self.reset_at),
            note: Some("ข้อมูลจำลอง (mock) — ยังไม่ได้เชื่อมต่อบัญชีจริง".to_string()),
        }
    }
}
