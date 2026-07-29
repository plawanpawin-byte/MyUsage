use chrono::{DateTime, Local};

pub mod local_cli;
pub mod mock;

/// A single point-in-time reading of an AI account's usage quota.
#[derive(Debug, Clone)]
pub struct UsageSnapshot {
    /// Stable machine id, e.g. "claude_pro".
    pub id: &'static str,
    /// Human readable name shown on the card, e.g. "Claude Pro".
    pub display_name: String,
    /// Accent color used for the icon badge (RGB).
    pub color: [u8; 3],
    /// 0.0..=100.0, where 100.0 means the quota is fully exhausted.
    pub percent_used: f32,
    /// When the quota window resets, if known.
    pub reset_at: Option<DateTime<Local>>,
    /// Optional short note surfaced in the UI, e.g. data-source caveats
    /// or error messages when a real provider fails to fetch.
    pub note: Option<String>,
}

impl UsageSnapshot {
    pub fn percent_left(&self) -> f32 {
        (100.0 - self.percent_used).clamp(0.0, 100.0)
    }
}

/// Anything that can report AI usage quota implements this. Swap in a real
/// implementation once you have credentials/an endpoint for your account —
/// see `local_cli.rs` for the extension point and README.md for context on
/// why there is no official public API to call by default.
pub trait UsageProvider: Send {
    fn fetch(&mut self) -> UsageSnapshot;
}
