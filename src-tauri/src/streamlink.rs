use crate::process_manager::ProcessManager;
use log::info;
use std::sync::Mutex;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// Cache VLC detection result with Mutex for updateable caching
static VLC_CACHE: Mutex<Option<(bool, std::time::SystemTime)>> = Mutex::new(None);

// Cache duration in seconds for VLC detection
const CACHE_DURATION_SECS: u64 = 30;

#[derive(Debug)]
pub struct StreamlinkManager {
    process_manager: ProcessManager,
}

impl StreamlinkManager {
    pub fn new() -> Self {
        Self {
            process_manager: ProcessManager::new(),
        }
    }

    pub fn start_stream(&mut self, url: &str, quality: &str) -> Result<(), String> {
        info!("StreamlinkManager: Starting stream {url} with quality {quality}");
        self.process_manager.start_stream(url, quality)
    }

    pub fn stop_stream(&mut self) -> bool {
        info!("StreamlinkManager: Stopping current stream");
        self.process_manager.stop_stream()
    }

    #[allow(dead_code)]
    pub fn is_streaming(&self) -> bool {
        self.process_manager.is_streaming()
    }

    #[allow(dead_code)]
    pub fn get_current_url(&self) -> Option<&String> {
        self.process_manager.get_current_url()
    }

    // Centralized and optimized VLC detection with time-based caching
    pub fn is_vlc_running(&self) -> bool {
        #[cfg(windows)]
        {
            // Check if we have cached result within valid duration
            if let Ok(cache) = VLC_CACHE.lock() {
                if let Some((vlc_available, timestamp)) = *cache {
                    if let Ok(elapsed) = timestamp.elapsed() {
                        if elapsed.as_secs() < CACHE_DURATION_SECS {
                            return vlc_available;
                        }
                    }
                }
            }

            // Perform fresh VLC detection
            let is_running = match std::process::Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq vlc.exe"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    output_str.contains("vlc.exe")
                },
                Err(e) => {
                    log::warn!("Failed to execute tasklist command for VLC detection: {}", e);
                    false
                }
            };
            
            // Update cache with fresh result
            if let Ok(mut cache) = VLC_CACHE.lock() {
                *cache = Some((is_running, std::time::SystemTime::now()));
            }
            
            return is_running;
        }

        #[cfg(not(windows))]
        {
            // Check if we have cached result within valid duration
            if let Ok(cache) = VLC_CACHE.lock() {
                if let Some((vlc_available, timestamp)) = *cache {
                    if let Ok(elapsed) = timestamp.elapsed() {
                        if elapsed.as_secs() < CACHE_DURATION_SECS {
                            return vlc_available;
                        }
                    }
                }
            }

            let is_running = match std::process::Command::new("pgrep")
                .arg("vlc")
                .output()
            {
                Ok(output) => {
                    !output.stdout.is_empty()
                },
                Err(e) => {
                    log::warn!("Failed to execute pgrep command for VLC detection: {}", e);
                    false
                }
            };

            // Update cache with fresh result
            if let Ok(mut cache) = VLC_CACHE.lock() {
                *cache = Some((is_running, std::time::SystemTime::now()));
            }

            return is_running;
        }
    }

    // Check if a stream is likely to be working by doing a quick streamlink test
    #[allow(dead_code)]
    pub fn test_stream_availability(&self, url: &str) -> bool {
        use std::process::{Command, Stdio};
        
        let channel = match crate::twitch::TwitchValidator::extract_channel_name(url) {
            Some(c) => c,
            None => return false,
        };

        let mut cmd = Command::new("streamlink");
        cmd.arg(format!("https://www.twitch.tv/{}", channel))
           .arg("--stream-info")
           .arg("--json")
           .stdout(Stdio::piped())
           .stderr(Stdio::null());

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    // Check if the JSON contains stream information
                    output_str.contains("\"streams\"") && !output_str.contains("\"streams\": {}")
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    log::warn!("Streamlink test failed for channel {}: {}", channel, stderr);
                    false
                }
            }
            Err(e) => {
                log::warn!("Failed to execute streamlink test for channel {}: {}", channel, e);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streamlink_manager_creation() {
        let manager = StreamlinkManager::new();
        assert!(!manager.is_streaming());
        assert!(manager.get_current_url().is_none());
    }

    #[test] 
    fn test_vlc_detection() {
        let manager = StreamlinkManager::new();
        // This will depend on whether VLC is actually running
        // Just test that it doesn't panic
        let _is_running = manager.is_vlc_running();
    }
}
