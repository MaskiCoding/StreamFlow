use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};

// Re-export shared types for convenience
pub use crate::types::{SavedStream, StreamStatus};

// Optimized settings structure - only essential data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_quality: String,
    pub quick_streams: Vec<SavedStream>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_quality: "best".to_string(),
            quick_streams: vec![],
        }
    }
}

impl Settings {
    // Get the app data directory path
    fn get_app_data_dir() -> Result<std::path::PathBuf> {
        let app_data =
            dirs::data_dir().ok_or_else(|| anyhow::anyhow!("Could not find AppData directory"))?;
        Ok(app_data.join("StreamFlow-Tauri"))
    }

    // Get the full path to settings file
    fn get_settings_path() -> Result<std::path::PathBuf> {
        let app_dir = Self::get_app_data_dir()?;
        Ok(app_dir.join("settings.json"))
    }

    // Load settings from file with performance optimizations
    pub fn load() -> Result<Self> {
        let settings_path = Self::get_settings_path()?;

        if !settings_path.exists() {
            log::info!(
                "Settings file not found, creating with defaults: {:?}",
                settings_path
            );
            let default_settings = Self::default();
            default_settings.save()?;
            return Ok(default_settings);
        }

        // Read and parse in one go to reduce I/O
        let contents = std::fs::read_to_string(&settings_path)
            .map_err(|e| anyhow::anyhow!("Failed to read settings file: {}", e))?;

        let settings: Settings = serde_json::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse settings JSON: {}", e))?;

        log::info!("Settings loaded successfully from: {:?}", settings_path);
        Ok(settings)
    }

    // Save settings to file with atomic write for data integrity
    pub fn save(&self) -> Result<()> {
        let app_dir = Self::get_app_data_dir()?;

        // Ensure directory exists
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir)
                .map_err(|e| anyhow::anyhow!("Failed to create app directory: {}", e))?;
            log::info!("Created app data directory: {:?}", app_dir);
        }

        let settings_path = Self::get_settings_path()?;
        let temp_path = settings_path.with_extension("json.tmp");

        // Serialize with pretty formatting for better readability
        let json_content = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize settings: {}", e))?;

        // Atomic write: write to temp file first, then rename
        std::fs::write(&temp_path, json_content)
            .map_err(|e| anyhow::anyhow!("Failed to write temp settings file: {}", e))?;

        std::fs::rename(&temp_path, &settings_path)
            .map_err(|e| anyhow::anyhow!("Failed to move temp settings file: {}", e))?;

        log::info!("Settings saved successfully to: {:?}", settings_path);
        Ok(())
    }

    // Add a new quick stream, maintaining the 4-stream limit
    #[allow(dead_code)]
    pub fn add_quick_stream(&mut self, stream: SavedStream) {
        if self.quick_streams.len() >= 4 {
            // Remove the oldest stream (first in the list)
            self.quick_streams.remove(0);
        }
        self.quick_streams.push(stream);
    }

    // Remove a quick stream by index
    #[allow(dead_code)]
    pub fn remove_quick_stream(&mut self, index: usize) -> bool {
        if index < self.quick_streams.len() {
            self.quick_streams.remove(index);
            true
        } else {
            false
        }
    }

    // Update stream status by URL
    #[allow(dead_code)]
    pub fn update_stream_status(&mut self, url: &str, status: StreamStatus) {
        for stream in &mut self.quick_streams {
            if stream.url == url {
                stream.status = status.clone();
                stream.last_checked = Some(Utc::now());
                break;
            }
        }
    }

    // Get stream by URL
    #[allow(dead_code)]
    pub fn get_stream_by_url(&self, url: &str) -> Option<&SavedStream> {
        self.quick_streams.iter().find(|stream| stream.url == url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.default_quality, "best");
        assert!(settings.quick_streams.is_empty());
    }

    #[test]
    fn test_add_quick_stream() {
        let mut settings = Settings::default();

        let stream = SavedStream {
            name: "Test Stream".to_string(),
            url: "https://www.twitch.tv/test".to_string(),
            status: StreamStatus::Unknown,
            last_checked: None,
        };

        settings.add_quick_stream(stream.clone());
        assert_eq!(settings.quick_streams.len(), 1);
        assert_eq!(settings.quick_streams[0].name, "Test Stream");
    }

    #[test]
    fn test_quick_stream_limit() {
        let mut settings = Settings::default();

        // Add 5 streams (more than the 4 limit)
        for i in 0..5 {
            let stream = SavedStream {
                name: format!("Stream {}", i),
                url: format!("https://www.twitch.tv/test{}", i),
                status: StreamStatus::Unknown,
                last_checked: None,
            };
            settings.add_quick_stream(stream);
        }

        // Should only have 4 streams, and the first one should be removed
        assert_eq!(settings.quick_streams.len(), 4);
        assert_eq!(settings.quick_streams[0].name, "Stream 1"); // Stream 0 was removed
        assert_eq!(settings.quick_streams[3].name, "Stream 4");
    }

    #[test]
    fn test_remove_quick_stream() {
        let mut settings = Settings::default();

        let stream = SavedStream {
            name: "Test Stream".to_string(),
            url: "https://www.twitch.tv/test".to_string(),
            status: StreamStatus::Unknown,
            last_checked: None,
        };

        settings.add_quick_stream(stream);
        assert_eq!(settings.quick_streams.len(), 1);

        assert!(settings.remove_quick_stream(0));
        assert!(settings.quick_streams.is_empty());

        // Try to remove from empty list
        assert!(!settings.remove_quick_stream(0));
    }

    #[test]
    fn test_update_stream_status() {
        let mut settings = Settings::default();

        let stream = SavedStream {
            name: "Test Stream".to_string(),
            url: "https://www.twitch.tv/test".to_string(),
            status: StreamStatus::Unknown,
            last_checked: None,
        };

        settings.add_quick_stream(stream);

        settings.update_stream_status("https://www.twitch.tv/test", StreamStatus::Online);

        let updated_stream = &settings.quick_streams[0];
        assert_eq!(updated_stream.status, StreamStatus::Online);
        assert!(updated_stream.last_checked.is_some());
    }

    #[test]
    fn test_get_stream_by_url() {
        let mut settings = Settings::default();

        let stream = SavedStream {
            name: "Test Stream".to_string(),
            url: "https://www.twitch.tv/test".to_string(),
            status: StreamStatus::Unknown,
            last_checked: None,
        };

        settings.add_quick_stream(stream);

        let found = settings.get_stream_by_url("https://www.twitch.tv/test");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Stream");

        let not_found = settings.get_stream_by_url("https://www.twitch.tv/notfound");
        assert!(not_found.is_none());
    }
}
