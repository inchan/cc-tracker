//! File watching for automatic prompt capture
//!
//! Monitors directories for new prompt files and captures them automatically.

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use crate::capture::CaptureService;
use crate::database::Database;
use crate::{PromptTrackingError, Result};

/// File watcher for automatic prompt capture
pub struct FileWatcher {
    watch_path: PathBuf,
    watcher: Option<RecommendedWatcher>,
    receiver: Option<Receiver<notify::Result<Event>>>,
    capture_service: CaptureService,
}

/// Watcher configuration
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    pub watch_path: PathBuf,
    pub recursive: bool,
    pub file_extensions: Vec<String>,
    pub similarity_threshold: f64,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            watch_path: PathBuf::from("."),
            recursive: true,
            file_extensions: vec![
                "txt".to_string(),
                "md".to_string(),
                "prompt".to_string(),
            ],
            similarity_threshold: 0.95,
        }
    }
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new(config: WatcherConfig) -> Result<Self> {
        let capture_service = CaptureService::new(config.similarity_threshold);

        Ok(Self {
            watch_path: config.watch_path,
            watcher: None,
            receiver: None,
            capture_service,
        })
    }

    /// Start watching for file changes
    pub fn start(&mut self) -> Result<()> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| {
            PromptTrackingError::IoError(std::io::Error::other(
                format!("Failed to create watcher: {}", e),
            ))
        })?;

        watcher
            .watch(&self.watch_path, RecursiveMode::Recursive)
            .map_err(|e| {
                PromptTrackingError::IoError(std::io::Error::other(
                    format!("Failed to watch path: {}", e),
                ))
            })?;

        self.watcher = Some(watcher);
        self.receiver = Some(rx);

        Ok(())
    }

    /// Stop watching
    pub fn stop(&mut self) {
        self.watcher = None;
        self.receiver = None;
    }

    /// Process pending events and capture prompts
    pub fn process_events(&self, db: &Database) -> Result<Vec<String>> {
        let mut captured_ids = Vec::new();

        if let Some(ref rx) = self.receiver {
            // Process all pending events
            while let Ok(event_result) = rx.try_recv() {
                if let Ok(event) = event_result {
                    if let Some(id) = self.handle_event(&event, db)? {
                        captured_ids.push(id);
                    }
                }
            }
        }

        Ok(captured_ids)
    }

    /// Handle a single file event
    fn handle_event(&self, event: &Event, db: &Database) -> Result<Option<String>> {
        // Only process create and modify events
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {}
            _ => return Ok(None),
        }

        for path in &event.paths {
            // Check if it's a file with valid extension
            if !self.is_valid_file(path) {
                continue;
            }

            // Try to capture the prompt
            match self.capture_service.capture_from_file(path) {
                Ok(prompt) => {
                    // Check for duplicates
                    if db.find_by_hash(&prompt.content_hash)?.is_some() {
                        continue;
                    }

                    // Save to database
                    db.create_prompt(&prompt)?;
                    return Ok(Some(prompt.id));
                }
                Err(_) => continue,
            }
        }

        Ok(None)
    }

    /// Check if file has valid extension
    fn is_valid_file(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            return ext_str == "txt" || ext_str == "md" || ext_str == "prompt";
        }

        false
    }

    /// Get watch path
    pub fn watch_path(&self) -> &Path {
        &self.watch_path
    }

    /// Check if watcher is running
    pub fn is_running(&self) -> bool {
        self.watcher.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();
        assert!(config.recursive);
        assert_eq!(config.similarity_threshold, 0.95);
    }

    #[test]
    fn test_file_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = WatcherConfig {
            watch_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let watcher = FileWatcher::new(config);
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_is_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = WatcherConfig {
            watch_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let watcher = FileWatcher::new(config).unwrap();

        // Create test files
        let valid_path = temp_dir.path().join("test.txt");
        std::fs::write(&valid_path, "test").unwrap();
        assert!(watcher.is_valid_file(&valid_path));

        let invalid_path = temp_dir.path().join("test.exe");
        std::fs::write(&invalid_path, "test").unwrap();
        assert!(!watcher.is_valid_file(&invalid_path));
    }

    #[test]
    fn test_watcher_start_stop() {
        let temp_dir = TempDir::new().unwrap();
        let config = WatcherConfig {
            watch_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut watcher = FileWatcher::new(config).unwrap();
        assert!(!watcher.is_running());

        watcher.start().unwrap();
        assert!(watcher.is_running());

        watcher.stop();
        assert!(!watcher.is_running());
    }
}
