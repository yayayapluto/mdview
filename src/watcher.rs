use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use anyhow::{Context, Result};
use eframe::egui;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

/// Watches the parent directory of a file so atomic saves (tmp file + rename)
/// are still reported.
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<()>,
}

impl FileWatcher {
    pub fn new(path: &Path, ctx: egui::Context) -> Result<Self> {
        let target = path
            .canonicalize()
            .with_context(|| format!("cannot resolve {}", path.display()))?;
        let parent = target
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            let Ok(event) = res else { return };
            if event.paths.iter().any(|p| p == &target) {
                let _ = tx.send(());
                ctx.request_repaint();
            }
        })
        .context("cannot create file watcher")?;

        watcher
            .watch(&parent, RecursiveMode::NonRecursive)
            .with_context(|| format!("cannot watch {}", parent.display()))?;

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    /// True if at least one change event arrived since the last call.
    pub fn changed(&self) -> bool {
        let mut changed = false;
        while self.rx.try_recv().is_ok() {
            changed = true;
        }
        changed
    }
}

/// Debounce interval for bursty editor saves.
pub const DEBOUNCE: Duration = Duration::from_millis(200);
