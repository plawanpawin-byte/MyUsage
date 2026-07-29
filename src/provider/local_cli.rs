use std::path::PathBuf;

use super::{mock::MockProvider, UsageProvider, UsageSnapshot};

/// Best-effort provider for CLI tools that keep a local session on disk
/// (e.g. the Codex CLI's `~/.codex` folder or Claude Code's `~/.claude`
/// folder).
///
/// Neither OpenAI nor Anthropic publish a documented, stable local file
/// format or public endpoint for "percent of quota used" on consumer plans
/// (ChatGPT Plus / Claude Pro), so this intentionally does NOT scrape or
/// guess at undocumented internals that could break silently on the next
/// CLI update.
///
/// What it does instead: detect whether a local session for the tool
/// exists, report that honestly as a note in the UI, and fall back to the
/// simulated numbers from `MockProvider` for the percentage/countdown. This
/// struct is the intended extension point — once you have a real endpoint
/// and credentials for your own account, replace the body of `fetch()` with
/// an actual HTTP call and drop the fallback.
pub struct LocalCliProvider {
    id: &'static str,
    display_name: String,
    color: [u8; 3],
    session_dir: PathBuf,
    fallback: MockProvider,
}

impl LocalCliProvider {
    pub fn new(
        id: &'static str,
        display_name: &str,
        color: [u8; 3],
        session_dir: PathBuf,
        fallback: MockProvider,
    ) -> Self {
        Self {
            id,
            display_name: display_name.to_string(),
            color,
            session_dir,
            fallback,
        }
    }

    fn has_local_session(&self) -> bool {
        self.session_dir.is_dir()
    }
}

impl UsageProvider for LocalCliProvider {
    fn fetch(&mut self) -> UsageSnapshot {
        let mut snap = self.fallback.fetch();
        snap.id = self.id;
        snap.display_name = self.display_name.clone();
        snap.color = self.color;
        snap.note = Some(if self.has_local_session() {
            format!(
                "พบ session ของ CLI ที่ {} แต่ยังไม่มี API สาธารณะสำหรับดึง % โควตา — แสดงค่าจำลอง",
                self.session_dir.display()
            )
        } else {
            format!(
                "ไม่พบ session ที่ {} — แสดงค่าจำลอง",
                self.session_dir.display()
            )
        });
        snap
    }
}
