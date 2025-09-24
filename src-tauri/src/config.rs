use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::error::{StreamFlowError, StreamFlowResult};
use crate::util::helpers::{get_config_dir, is_portable};

/// Main configuration structure for StreamFlow-Tauri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default stream quality
    pub default_quality: String,
    /// Quick access streams (max 4)
    pub quick_streams: Vec<SavedStream>,
    /// Window settings
    pub window: WindowConfig,
    /// Application behavior settings
    pub behavior: BehaviorConfig,
    /// Advanced settings
    pub advanced: AdvancedConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_quality: "best".to_string(),
            quick_streams: Vec::new(),
            window: WindowConfig::default(),
            behavior: BehaviorConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedStream {
    pub name: String,
    pub url: String,
    pub status: StreamStatus,
    pub last_checked: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamStatus {
    Online,
    Offline,
    Unknown,
}

impl Default for StreamStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub maximized: bool,
    pub always_on_top: bool,
    pub decorations: bool,
    pub transparent: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: Some(1200),
            height: Some(800),
            maximized: false,
            always_on_top: false,
            decorations: true,
            transparent: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    pub auto_check_streams: bool,
    pub check_interval_minutes: u32,
    pub start_minimized: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            minimize_to_tray: false,
            close_to_tray: false,
            auto_check_streams: true,
            check_interval_minutes: 5,
            start_minimized: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub vlc_path: Option<String>,
    pub streamlink_path: Option<String>,
    pub proxy_url: Option<String>,
    pub disable_hardware_accel: bool,
    pub debug_mode: bool,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            vlc_path: None,
            streamlink_path: None,
            proxy_url: None,
            disable_hardware_accel: false,
            debug_mode: false,
        }
    }
}

impl Config {
    /// Initialize default configuration
    pub fn init() {
        let config_path = Self::get_config_path();
        if !config_path.exists() {
            log::info!("Creating default configuration at: {:?}", config_path);
            let config = Self::default();
            let _ = config.save();
        }
    }

    /// Get the configuration file path
    pub fn get_config_path() -> PathBuf {
        if is_portable() {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.join("config.json")))
                .unwrap_or_else(|| PathBuf::from("config.json"))
        } else {
            get_config_dir().join("config.json")
        }
    }

    /// Load configuration from file
    pub fn load() -> StreamFlowResult<Self> {
        let path = Self::get_config_path();

        if !path.exists() {
            log::warn!("Configuration file not found at: {:?}, creating default", path);
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let contents = std::fs::read_to_string(&path)?;
        let config: Self = serde_json::from_str(&contents)?;

        log::info!("Configuration loaded from: {:?}", path);
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> StreamFlowResult<()> {
        let path = Self::get_config_path();
        let parent = path.parent().ok_or_else(|| {
            StreamFlowError::ConfigError("Invalid config path".to_string())
        })?;

        // Create directory if it doesn't exist
        std::fs::create_dir_all(parent)?;

        // Write atomically using a temporary file
        let temp_path = path.with_extension("json.tmp");
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&temp_path, contents)?;
        std::fs::rename(&temp_path, &path)?;

        log::debug!("Configuration saved to: {:?}", path);
        Ok(())
    }

    /// Get a mutable reference to quick streams
    pub fn get_quick_streams(&self) -> &[SavedStream] {
        &self.quick_streams
    }

    /// Add a quick stream (maintains max 4)
    pub fn add_quick_stream(&mut self, stream: SavedStream) -> StreamFlowResult<()> {
        // Remove if URL already exists
        self.quick_streams.retain(|s| s.url != stream.url);

        // Add to front
        self.quick_streams.insert(0, stream);

        // Keep only the first 4
        self.quick_streams.truncate(4);

        self.save()
    }

    /// Remove a quick stream by index
    pub fn remove_quick_stream(&mut self, index: usize) -> StreamFlowResult<()> {
        if index < self.quick_streams.len() {
            self.quick_streams.remove(index);
            self.save()
        } else {
            Err(StreamFlowError::ConfigError("Invalid stream index".to_string()))
        }
    }

    /// Update stream status
    pub fn update_stream_status(&mut self, url: &str, status: StreamStatus) -> StreamFlowResult<()> {
        if let Some(stream) = self.quick_streams.iter_mut().find(|s| s.url == url) {
            stream.status = status;
            stream.last_checked = Some(chrono::Utc::now());
            self.save()
        } else {
            Err(StreamFlowError::ConfigError("Stream not found".to_string()))
        }
    }

    /// Get environment-specific overrides
    pub fn get_env_overrides() -> HashMap<String, String> {
        let mut overrides = HashMap::new();

        // Check environment variables
        if let Ok(proxy) = std::env::var("STREAMFLOW_PROXY") {
            overrides.insert("proxy_url".to_string(), proxy);
        }

        if let Ok(quality) = std::env::var("STREAMFLOW_QUALITY") {
            overrides.insert("default_quality".to_string(), quality);
        }

        if let Ok(vlc_path) = std::env::var("STREAMFLOW_VLC_PATH") {
            overrides.insert("vlc_path".to_string(), vlc_path);
        }

        overrides
    }
}

// Tauri commands for configuration management
#[tauri::command]
pub async fn get_config() -> StreamFlowResult<Config> {
    Config::load()
}

#[tauri::command]
pub async fn set_config(config: Config) -> StreamFlowResult<String> {
    config.save()?;
    Ok("Configuration saved successfully".to_string())
}

#[tauri::command]
pub async fn read_config_file() -> StreamFlowResult<String> {
    let config = Config::load()?;
    Ok(serde_json::to_string_pretty(&config)?)
}

#[tauri::command]
pub async fn write_config_file(contents: String) -> StreamFlowResult<String> {
    let config: Config = serde_json::from_str(&contents)?;
    config.save()?;
    Ok("Configuration file written successfully".to_string())
}

#[tauri::command]
pub async fn default_config() -> StreamFlowResult<Config> {
    Ok(Config::default())
}