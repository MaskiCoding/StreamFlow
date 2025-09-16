#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;
use tauri::{Manager, State};
use scraper::{Html, Selector};

mod twitch;
mod settings;
mod streamlink;
mod process_manager;

use settings::{Settings, SavedStream, StreamStatus};
use streamlink::StreamlinkManager;
use crate::process_manager::ProcessManager;

// Application state to manage across the frontend and backend
#[derive(Debug)]
pub struct AppState {
    streamlink_manager: Mutex<StreamlinkManager>,
    settings: Mutex<Settings>,
    current_stream: Mutex<Option<String>>,
}

impl AppState {
    pub fn new() -> Self {
        let settings = Settings::load().unwrap_or_else(|e| {
            log::warn!("Failed to load settings: {}. Using defaults.", e);
            Settings::default()
        });

        Self {
            streamlink_manager: Mutex::new(StreamlinkManager::new()),
            settings: Mutex::new(settings),
            current_stream: Mutex::new(None),
        }
    }
}

// Tauri commands for frontend-backend communication
#[tauri::command]
async fn start_stream(
    url: String,
    quality: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("Starting stream: {} with quality: {}", url, quality);
    
    let mut manager = state.streamlink_manager.lock().unwrap();
    let mut current = state.current_stream.lock().unwrap();
    
    match manager.start_stream(&url, &quality) {
        Ok(()) => {
            *current = Some(url.clone());
            Ok(format!("Started stream: {}", url))
        }
        Err(e) => {
            log::error!("Failed to start stream: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
async fn stop_stream(state: State<'_, AppState>) -> Result<String, String> {
    log::info!("Stopping current stream");
    
    let mut manager = state.streamlink_manager.lock().unwrap();
    let mut current = state.current_stream.lock().unwrap();
    
    if manager.stop_stream() {
        *current = None;
        Ok("Stream stopped successfully".to_string())
    } else {
        Err("No stream to stop or failed to stop".to_string())
    }
}

#[tauri::command]
async fn get_current_stream(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let current = state.current_stream.lock().unwrap();
    Ok(current.clone())
}

#[tauri::command]
async fn is_vlc_running(state: State<'_, AppState>) -> Result<bool, String> {
    let manager = state.streamlink_manager.lock().unwrap();
    Ok(manager.is_vlc_running())
}

#[tauri::command]
async fn validate_twitch_url(url: String) -> Result<bool, String> {
    Ok(twitch::TwitchValidator::is_valid_url(&url))
}

#[tauri::command]
async fn extract_channel_name(url: String) -> Result<Option<String>, String> {
    Ok(twitch::TwitchValidator::extract_channel_name(&url))
}

#[tauri::command]
async fn normalize_twitch_url(url: String) -> Result<String, String> {
    Ok(twitch::TwitchValidator::normalize_url(&url))
}

#[tauri::command]
async fn load_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock().unwrap();
    Ok(settings.clone())
}

#[tauri::command]
async fn save_settings(
    new_settings: Settings,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut settings = state.settings.lock().unwrap();
    *settings = new_settings.clone();
    
    match new_settings.save() {
        Ok(()) => Ok("Settings saved successfully".to_string()),
        Err(e) => {
            log::error!("Failed to save settings: {}", e);
            Err(format!("Failed to save settings: {}", e))
        }
    }
}

#[tauri::command]
async fn add_quick_stream(
    name: String,
    url: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut settings = state.settings.lock().unwrap();
    
    let new_stream = SavedStream {
        name,
        url,
        status: StreamStatus::Unknown,
        last_checked: None,
    };
    
    // Add to quick streams (max 4)
    if settings.quick_streams.len() < 4 {
        settings.quick_streams.push(new_stream);
    } else {
        // Replace the oldest one
        settings.quick_streams[0] = new_stream;
        settings.quick_streams.rotate_left(1);
    }
    
    match settings.save() {
        Ok(()) => Ok("Quick stream added successfully".to_string()),
        Err(e) => Err(format!("Failed to save quick stream: {}", e))
    }
}

#[tauri::command]
async fn remove_quick_stream(
    index: usize,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut settings = state.settings.lock().unwrap();
    
    if index < settings.quick_streams.len() {
        settings.quick_streams.remove(index);
        match settings.save() {
            Ok(()) => Ok("Quick stream removed successfully".to_string()),
            Err(e) => Err(format!("Failed to save settings: {}", e))
        }
    } else {
        Err("Invalid quick stream index".to_string())
    }
}

// Shared function to create HTTP client with proper configuration
fn create_http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("StreamFlow/1.0 (+https://streamflow-tauri.com)")
        .build()
        .map_err(|e| format!("HTTP client initialization failed: {}", e))
}

// Cleanup function to ensure all processes are terminated on application exit
fn cleanup_processes(app_state: &AppState) {
    log::info!("Performing application cleanup...");

    // Stop any active streams
    let mut manager = app_state.streamlink_manager.lock().unwrap();
    if manager.is_streaming() {
        log::info!("Stopping active stream during cleanup");
        let _ = manager.stop_stream();
    }

    // Kill any remaining VLC processes
    let process_manager = ProcessManager::new();
    process_manager.kill_vlc_processes();

    log::info!("Application cleanup completed");
}

// Shared function to check stream status for a given channel
async fn check_twitch_stream_status(channel: &str) -> Result<String, String> {
    log::debug!("Checking stream status for channel: {}", channel);
    
    // Create a new client with timeout and user agent
    let client = create_http_client()?;
    
    match client.get(&format!("https://www.twitch.tv/{}", channel)).send().await {
        Ok(response) => {
            log::debug!("Received response for channel {}: status {}", channel, response.status());
            
            if response.status().is_success() {
                let html_text = match response.text().await {
                    Ok(text) => text,
                    Err(e) => {
                        log::error!("Failed to read Twitch response for {}: {}", channel, e);
                        return Err(format!("Network error reading response for {}: {}", channel, e));
                    }
                };
                
                log::debug!("Successfully retrieved HTML content for channel {} ({} bytes)", channel, html_text.len());
                
                // Try to parse with scraper for more reliable detection
                // Html::parse_document returns Html directly, not Result
                let document = Html::parse_document(&html_text);
                log::debug!("Successfully parsed HTML document for channel {}", channel);

                log::debug!("Successfully parsed HTML document for channel {}", channel);

                // Look for the live indicator in the parsed HTML
                let live_selector = Selector::parse("span.live-indicator").map_err(|e| {
                    log::warn!("Failed to parse live indicator selector for channel {}: {}", channel, e);
                    format!("Failed to parse HTML selector for live indicator: {}", e)
                })?;

                // Check if any elements match the selector
                if document.select(&live_selector).next().is_some() {
                    log::info!("Channel {} is Online (found live indicator element)", channel);
                    Ok("Online".to_string())
                } else {
                    log::debug!("Live indicator element not found for channel {}, checking with string matching", channel);

                    // Fallback to basic string search if scraper doesn't find anything
                    if html_text.contains("isLiveBroadcast\":true") || html_text.contains("Live on Twitch") {
                        log::info!("Channel {} is Online (found live broadcast string)", channel);
                        Ok("Online".to_string())
                    } else if html_text.contains("isLiveBroadcast\":false") || html_text.contains("offline") {
                        log::info!("Channel {} is Offline", channel);
                        Ok("Offline".to_string())
                    } else {
                        log::info!("Channel {} status is Unknown", channel);
                        Ok("Unknown".to_string())
                    }
                }
            } else {
                // Add structured error for failed status checks
                log::warn!("Status determination for channel '{}' was inconclusive (HTTP {})", channel, response.status());
                Ok("Unknown".to_string())
            }
        }
        Err(e) => {
            log::error!("Failed to check stream status for channel {}: {}", channel, e);
            Err(format!("Failed to check stream status: {}", e))
        }
    }
}

#[tauri::command]
async fn check_stream_status(
    url: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("Checking stream status for: {}", url);
    
    let channel = match twitch::TwitchValidator::extract_channel_name(&url) {
        Some(c) => c,
        None => return Err("Invalid Twitch URL".to_string()),
    };
    
    // Use the shared function to check stream status
    check_twitch_stream_status(&channel).await
}

#[tauri::command]
#[allow(dead_code)]
#[allow(unused_variables)]
async fn check_all_quick_streams_status(
    _state: State<'_, AppState>,
) -> Result<Vec<(usize, String)>, String> {
    // This function currently doesn't use the state, but keeping it for future compatibility
    // For now, we just return empty results to maintain the API contract
    Ok(Vec::new())
}

#[tauri::command]
#[allow(dead_code)]
async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}



#[tauri::command]
async fn update_quick_stream_status(
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("Updating status for all quick streams");
    
    // Get the streams to check without holding the lock
    let streams_to_check: Vec<(usize, String, String)> = {
        let settings = state.settings.lock().unwrap();
        settings.quick_streams.iter()
            .enumerate()
            .filter_map(|(i, stream)| {
                twitch::TwitchValidator::extract_channel_name(&stream.url)
                    .map(|channel| (i, stream.url.clone(), channel))
            })
            .collect()
    };
    
    if streams_to_check.is_empty() {
        return Ok("No streams to check".to_string());
    }
    
    let mut status_updates: Vec<(usize, settings::StreamStatus)> = Vec::new();
    
    // Check streams concurrently like the original StreamFlow
    let mut handles = Vec::new();
    
    for (index, url, channel) in streams_to_check {
        log::info!("Checking status for stream {}: {}", index, url);
        
        let handle = tokio::spawn(async move {
            // Use the shared function to check stream status
            match check_twitch_stream_status(&channel).await {
                Ok(status) => {
                    match status.as_str() {
                        "Online" => settings::StreamStatus::Online,
                        "Offline" => settings::StreamStatus::Offline,
                        _ => settings::StreamStatus::Unknown,
                    }
                },
                Err(e) => {
                    log::error!("Failed to check stream {} status: {}", index, e);
                    settings::StreamStatus::Unknown
                }
            }
        });
        
        handles.push((index, handle));
    }
    
    // Wait for all checks to complete
    for (index, handle) in handles {
        match handle.await {
            Ok(status) => {
                log::info!("Stream {} status: {:?}", index, status);
                status_updates.push((index, status));
            }
            Err(e) => {
                log::warn!("Status check failed for stream {}: {}", index, e);
                status_updates.push((index, settings::StreamStatus::Unknown));
            }
        }
    }
    
    // Apply all the status updates
    {
        let mut settings = state.settings.lock().unwrap();
        for (index, status) in status_updates {
            if index < settings.quick_streams.len() {
                settings.quick_streams[index].status = status;
                settings.quick_streams[index].last_checked = Some(chrono::Utc::now());
            }
        }
        
        match settings.save() {
            Ok(()) => {
                log::info!("Live status check completed");
                Ok(format!("Updated stream statuses"))
            },
            Err(e) => Err(format!("Failed to save settings: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_channel_name_valid_urls() {
        // Test valid Twitch URLs
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://www.twitch.tv/channel"), Some("channel".to_string()));
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://twitch.tv/channel"), Some("channel".to_string()));
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://m.twitch.tv/channel"), Some("channel".to_string()));
        assert_eq!(twitch::TwitchValidator::extract_channel_name("http://www.twitch.tv/channel"), Some("channel".to_string()));
    }

    #[test]
    fn test_extract_channel_name_valid_dashboard_urls() {
        // Test valid Twitch dashboard URLs
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://www.twitch.tv/dashboard/channel"), Some("channel".to_string()));
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://twitch.tv/dashboard/channel"), Some("channel".to_string()));
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://m.twitch.tv/dashboard/channel"), Some("channel".to_string()));
        assert_eq!(twitch::TwitchValidator::extract_channel_name("http://www.twitch.tv/dashboard/channel"), Some("channel".to_string()));
    }

    #[test]
    fn test_extract_channel_name_invalid_urls() {
        // Test invalid Twitch URLs
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://www.youtube.com/channel"), None);
        assert_eq!(twitch::TwitchValidator::extract_channel_name("not-a-url"), None);
        assert_eq!(twitch::TwitchValidator::extract_channel_name(""), None);
    }

    #[test]
    fn test_normalize_twitch_url() {
        // Test Twitch URL normalization
        assert_eq!(twitch::TwitchValidator::normalize_url("twitch.tv/channel"), "https://www.twitch.tv/channel");
        assert_eq!(twitch::TwitchValidator::normalize_url("www.twitch.tv/channel"), "https://www.twitch.tv/channel");
        assert_eq!(twitch::TwitchValidator::normalize_url("https://twitch.tv/channel"), "https://www.twitch.tv/channel");
        assert_eq!(twitch::TwitchValidator::normalize_url("https://www.twitch.tv/channel"), "https://www.twitch.tv/channel");
    }

    #[test]
    fn test_is_valid_twitch_url() {
        // Test valid Twitch URLs
        assert!(twitch::TwitchValidator::is_valid_url("https://www.twitch.tv/channel"));
        assert!(twitch::TwitchValidator::is_valid_url("https://twitch.tv/channel"));
        assert!(twitch::TwitchValidator::is_valid_url("https://m.twitch.tv/channel"));
        assert!(twitch::TwitchValidator::is_valid_url("http://www.twitch.tv/channel"));
        
        // Test valid Twitch dashboard URLs
        assert!(twitch::TwitchValidator::is_valid_url("https://www.twitch.tv/dashboard/channel"));
        assert!(twitch::TwitchValidator::is_valid_url("https://twitch.tv/dashboard/channel"));
        
        // Test invalid Twitch URLs
        assert!(!twitch::TwitchValidator::is_valid_url("https://www.youtube.com/channel"));
        assert!(!twitch::TwitchValidator::is_valid_url("not-a-url"));
        assert!(!twitch::TwitchValidator::is_valid_url(""));
    }

    #[test]
    fn test_stream_status_edge_cases() {
        // Test that the channel extraction handles various edge cases
        // These tests don't actually call the Twitch API, but verify the logic
        
        // Test valid channel names at boundary conditions
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://www.twitch.tv/abc"), Some("abc".to_string())); // 3 chars min
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://www.twitch.tv/abcdefghijklmnopqrstuvwxy"), Some("abcdefghijklmnopqrstuvwxy".to_string())); // 25 chars max
        
        // Test invalid channel names at boundary conditions
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://www.twitch.tv/ab"), None); // 2 chars - too short
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://www.twitch.tv/abcdefghijklmnopqrstuvwxyz"), None); // 26 chars - too long
        
        // Test channel names with valid special characters (underscores)
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://www.twitch.tv/user_name"), Some("user_name".to_string()));
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://www.twitch.tv/_username"), Some("_username".to_string()));
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://www.twitch.tv/username_"), Some("username_".to_string()));
        
        // Test channel names with invalid special characters
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://www.twitch.tv/user-name"), None); // hyphens not allowed
        assert_eq!(twitch::TwitchValidator::extract_channel_name("https://www.twitch.tv/user.name"), None); // periods not allowed
    }
}

fn main() {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Starting StreamFlow-Tauri v{}", env!("CARGO_PKG_VERSION"));

    // Initialize application state
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_stream,
            stop_stream,
            get_current_stream,
            is_vlc_running,
            validate_twitch_url,
            extract_channel_name,
            normalize_twitch_url,
            load_settings,
            save_settings,
            add_quick_stream,
            remove_quick_stream,
            get_app_version,
            check_stream_status,
            update_quick_stream_status
        ])
        .setup(|app| {
            // Setup window and initialize the application
            let window = app.get_webview_window("main").unwrap();

            // Set window properties
            let _ = window.set_title("StreamFlow");

            log::info!("Application setup completed");
            Ok(())
        })
        .on_window_event(|window, event| {
            // Handle window close event to ensure cleanup
            match event {
                tauri::WindowEvent::CloseRequested { .. } => {
                    log::info!("Application window close requested, performing cleanup...");
                    // Get the app state and perform cleanup
                    if let Some(app_handle) = window.app_handle().clone().try_state::<AppState>() {
                        cleanup_processes(&app_handle);
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
