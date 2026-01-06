use crate::error::{StreamFlowError, StreamFlowResult};
use crate::process_manager::ProcessManager;
use crate::settings::{SavedStream, Settings, StreamStatus};
use crate::streamlink::StreamlinkManager;
use crate::twitch::TwitchValidator;
use scraper::{Html, Selector};
use std::sync::Mutex;
use tauri::State;

/// Application state to manage across the frontend and backend
#[derive(Debug)]
pub struct AppState {
    pub streamlink_manager: Mutex<StreamlinkManager>,
    pub settings: Mutex<Settings>,
    pub current_stream: Mutex<Option<String>>,
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
pub async fn start_stream(
    url: String,
    quality: String,
    state: State<'_, AppState>,
) -> StreamFlowResult<String> {
    log::info!("Starting stream: {} with quality: {}", url, quality);

    let mut manager = state.streamlink_manager.lock().map_err(|_| {
        StreamFlowError::ProcessError("Failed to acquire stream manager lock".to_string())
    })?;
    let mut current = state.current_stream.lock().map_err(|_| {
        StreamFlowError::ProcessError("Failed to acquire current stream lock".to_string())
    })?;

    match manager.start_stream(&url, &quality) {
        Ok(()) => {
            *current = Some(url.clone());
            Ok(format!("Started stream: {}", url))
        }
        Err(e) => {
            log::error!("Failed to start stream: {}", e);
            Err(StreamFlowError::StreamError(e))
        }
    }
}

#[tauri::command]
pub async fn stop_stream(state: State<'_, AppState>) -> StreamFlowResult<String> {
    log::info!("Stopping current stream");

    let mut manager = state.streamlink_manager.lock().map_err(|_| {
        StreamFlowError::ProcessError("Failed to acquire stream manager lock".to_string())
    })?;
    let mut current = state.current_stream.lock().map_err(|_| {
        StreamFlowError::ProcessError("Failed to acquire current stream lock".to_string())
    })?;

    if manager.stop_stream() {
        *current = None;
        Ok("Stream stopped successfully".to_string())
    } else {
        Err(StreamFlowError::StreamError(
            "No stream to stop or failed to stop".to_string(),
        ))
    }
}

#[tauri::command]
pub async fn get_current_stream(state: State<'_, AppState>) -> StreamFlowResult<Option<String>> {
    let current = state.current_stream.lock().map_err(|_| {
        StreamFlowError::ProcessError("Failed to acquire current stream lock".to_string())
    })?;
    Ok(current.clone())
}

#[tauri::command]
pub async fn is_vlc_running(state: State<'_, AppState>) -> StreamFlowResult<bool> {
    let manager = state.streamlink_manager.lock().map_err(|_| {
        StreamFlowError::ProcessError("Failed to acquire stream manager lock".to_string())
    })?;
    Ok(manager.is_vlc_running())
}

#[tauri::command]
pub async fn validate_twitch_url(url: String) -> StreamFlowResult<bool> {
    Ok(TwitchValidator::is_valid_url(&url))
}

#[tauri::command]
pub async fn extract_channel_name(url: String) -> StreamFlowResult<Option<String>> {
    Ok(TwitchValidator::extract_channel_name(&url))
}

#[tauri::command]
pub async fn normalize_twitch_url(url: String) -> StreamFlowResult<String> {
    Ok(TwitchValidator::normalize_url(&url))
}

#[tauri::command]
pub async fn load_settings(state: State<'_, AppState>) -> StreamFlowResult<Settings> {
    let settings = state.settings.lock().map_err(|_| {
        StreamFlowError::ProcessError("Failed to acquire settings lock".to_string())
    })?;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn save_settings(
    new_settings: Settings,
    state: State<'_, AppState>,
) -> StreamFlowResult<String> {
    let mut settings = state.settings.lock().map_err(|_| {
        StreamFlowError::ProcessError("Failed to acquire settings lock".to_string())
    })?;
    *settings = new_settings.clone();

    match new_settings.save() {
        Ok(()) => Ok("Settings saved successfully".to_string()),
        Err(e) => {
            log::error!("Failed to save settings: {}", e);
            Err(StreamFlowError::ConfigError(format!(
                "Failed to save settings: {}",
                e
            )))
        }
    }
}

#[tauri::command]
pub async fn add_quick_stream(
    name: String,
    url: String,
    state: State<'_, AppState>,
) -> StreamFlowResult<String> {
    let mut settings = state.settings.lock().map_err(|_| {
        StreamFlowError::ProcessError("Failed to acquire settings lock".to_string())
    })?;

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
        Err(e) => Err(StreamFlowError::ConfigError(format!(
            "Failed to save quick stream: {}",
            e
        ))),
    }
}

#[tauri::command]
pub async fn remove_quick_stream(
    index: usize,
    state: State<'_, AppState>,
) -> StreamFlowResult<String> {
    let mut settings = state.settings.lock().map_err(|_| {
        StreamFlowError::ProcessError("Failed to acquire settings lock".to_string())
    })?;

    if index < settings.quick_streams.len() {
        settings.quick_streams.remove(index);
        match settings.save() {
            Ok(()) => Ok("Quick stream removed successfully".to_string()),
            Err(e) => Err(StreamFlowError::ConfigError(format!(
                "Failed to save settings: {}",
                e
            ))),
        }
    } else {
        Err(StreamFlowError::ValidationError(
            "Invalid quick stream index".to_string(),
        ))
    }
}

/// Shared function to create HTTP client with proper configuration
fn create_http_client() -> StreamFlowResult<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("StreamFlow-Tauri/1.0")
        .build()
        .map_err(|e| {
            StreamFlowError::NetworkError(format!("HTTP client initialization failed: {}", e))
        })
}

/// Cleanup function to ensure all processes are terminated on application exit
pub fn cleanup_processes(app_state: &AppState) {
    log::info!("Performing application cleanup...");

    // Stop any active streams
    let Ok(mut manager) = app_state.streamlink_manager.lock() else {
        log::error!("Failed to acquire stream manager lock during cleanup");
        return;
    };
    if manager.is_streaming() {
        log::info!("Stopping active stream during cleanup");
        let _ = manager.stop_stream();
    }

    // Kill any remaining VLC processes
    let process_manager = ProcessManager::new();
    process_manager.kill_vlc_processes();

    log::info!("Application cleanup completed");
}

/// Shared function to check stream status for a given channel
async fn check_twitch_stream_status(channel: &str) -> StreamFlowResult<String> {
    log::debug!("Checking stream status for channel: {}", channel);

    // Create a new client with timeout and user agent
    let client = create_http_client()?;

    match client
        .get(&format!("https://www.twitch.tv/{}", channel))
        .send()
        .await
    {
        Ok(response) => {
            log::debug!(
                "Received response for channel {}: status {}",
                channel,
                response.status()
            );

            if response.status().is_success() {
                let html_text = match response.text().await {
                    Ok(text) => text,
                    Err(e) => {
                        log::error!("Failed to read Twitch response for {}: {}", channel, e);
                        return Err(StreamFlowError::NetworkError(format!(
                            "Network error reading response for {}: {}",
                            channel, e
                        )));
                    }
                };

                log::debug!(
                    "Successfully retrieved HTML content for channel {} ({} bytes)",
                    channel,
                    html_text.len()
                );

                // Try to parse with scraper for more reliable detection
                let document = Html::parse_document(&html_text);
                log::debug!("Successfully parsed HTML document for channel {}", channel);

                // Look for the live indicator in the parsed HTML
                let live_selector = Selector::parse("span.live-indicator").map_err(|e| {
                    log::warn!(
                        "Failed to parse live indicator selector for channel {}: {}",
                        channel,
                        e
                    );
                    StreamFlowError::GenericError(format!(
                        "Failed to parse HTML selector for live indicator: {}",
                        e
                    ))
                })?;

                // Check if any elements match the selector
                if document.select(&live_selector).next().is_some() {
                    log::info!(
                        "Channel {} is Online (found live indicator element)",
                        channel
                    );
                    Ok("Online".to_string())
                } else {
                    log::debug!("Live indicator element not found for channel {}, checking with string matching", channel);

                    // Fallback to basic string search if scraper doesn't find anything
                    if html_text.contains("isLiveBroadcast\":true")
                        || html_text.contains("Live on Twitch")
                    {
                        log::info!(
                            "Channel {} is Online (found live broadcast string)",
                            channel
                        );
                        Ok("Online".to_string())
                    } else if html_text.contains("isLiveBroadcast\":false")
                        || html_text.contains("offline")
                    {
                        log::info!("Channel {} is Offline", channel);
                        Ok("Offline".to_string())
                    } else {
                        log::info!("Channel {} status is Unknown", channel);
                        Ok("Unknown".to_string())
                    }
                }
            } else {
                // Add structured error for failed status checks
                log::warn!(
                    "Status determination for channel '{}' was inconclusive (HTTP {})",
                    channel,
                    response.status()
                );
                Ok("Unknown".to_string())
            }
        }
        Err(e) => {
            log::error!(
                "Failed to check stream status for channel {}: {}",
                channel,
                e
            );
            Err(StreamFlowError::NetworkError(format!(
                "Failed to check stream status: {}",
                e
            )))
        }
    }
}

#[tauri::command]
pub async fn check_stream_status(url: String) -> StreamFlowResult<String> {
    log::info!("Checking stream status for: {}", url);

    let channel = match TwitchValidator::extract_channel_name(&url) {
        Some(c) => c,
        None => {
            return Err(StreamFlowError::ValidationError(
                "Invalid Twitch URL".to_string(),
            ))
        }
    };

    // Use the shared function to check stream status
    check_twitch_stream_status(&channel).await
}

#[tauri::command]
pub async fn get_app_version() -> StreamFlowResult<String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
pub async fn update_quick_stream_status(state: State<'_, AppState>) -> StreamFlowResult<String> {
    log::info!("Updating status for all quick streams");

    // Get the streams to check without holding the lock
    let streams_to_check: Vec<(usize, String, String)> = {
        let settings = state.settings.lock().map_err(|_| {
            StreamFlowError::ProcessError("Failed to acquire settings lock".to_string())
        })?;
        settings
            .quick_streams
            .iter()
            .enumerate()
            .filter_map(|(i, stream)| {
                TwitchValidator::extract_channel_name(&stream.url)
                    .map(|channel| (i, stream.url.clone(), channel))
            })
            .collect()
    };

    if streams_to_check.is_empty() {
        return Ok("No streams to check".to_string());
    }

    let mut status_updates: Vec<(usize, StreamStatus)> = Vec::new();

    // Check streams concurrently like the original StreamFlow
    let mut handles = Vec::new();

    for (index, url, channel) in streams_to_check {
        log::info!("Checking status for stream {}: {}", index, url);

        let handle = tokio::spawn(async move {
            // Use the shared function to check stream status
            match check_twitch_stream_status(&channel).await {
                Ok(status) => match status.as_str() {
                    "Online" => StreamStatus::Online,
                    "Offline" => StreamStatus::Offline,
                    _ => StreamStatus::Unknown,
                },
                Err(e) => {
                    log::error!("Failed to check stream {} status: {}", index, e);
                    StreamStatus::Unknown
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
                status_updates.push((index, StreamStatus::Unknown));
            }
        }
    }

    // Apply all the status updates
    {
        let mut settings = state.settings.lock().map_err(|_| {
            StreamFlowError::ProcessError("Failed to acquire settings lock".to_string())
        })?;
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
            }
            Err(e) => Err(StreamFlowError::ConfigError(format!(
                "Failed to save settings: {}",
                e
            ))),
        }
    }
}
