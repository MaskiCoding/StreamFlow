use log::{error, info, warn};
use std::process::{Child, Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

// Process management constants
const RETRY_STREAMS: &str = "3";
const TTV_LOL_PROXY_URL: &str = "https://eu.luminous.dev";

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug)]
pub struct ProcessManager {
    current_process: Option<Child>,
    current_url: Option<String>,
}

impl ProcessManager {
    pub const fn new() -> Self {
        Self {
            current_process: None,
            current_url: None,
        }
    }

    pub fn start_stream(&mut self, url: &str, quality: &str) -> Result<(), String> {
        // Stop any existing stream first
        self.stop_stream();

        info!("Starting stream: {url} with quality: {quality}");

        // Check if streamlink is available (cached result to avoid repeated checks)
        if !self.is_streamlink_available() {
            return Err(
                "Streamlink not found. Please install with: pip install streamlink".to_string(),
            );
        }

        // Extract channel name - avoid string allocation if possible
        let channel = crate::twitch::TwitchValidator::extract_channel_name(url)
            .ok_or_else(|| "Invalid Twitch URL".to_string())?;

        // Build command efficiently
        let mut cmd = Command::new("streamlink");

        // Add luminous.dev proxy for ad blocking (TTV LOL)
        cmd.arg("--twitch-proxy-playlist")
            .arg(TTV_LOL_PROXY_URL)
            .arg("--twitch-disable-ads");

        info!("Using luminous.dev ad-blocking proxy: {TTV_LOL_PROXY_URL}");

        // Add stream arguments in one go to reduce allocations
        cmd.arg(format!("https://www.twitch.tv/{channel}"))
            .arg(quality)
            .arg("--player-continuous-http")
            .arg("--retry-streams")
            .arg(RETRY_STREAMS)
            .arg("--twitch-low-latency");

        // Hide console window on Windows
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        // Configure stdio to capture output for debugging
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        info!("Executing streamlink command for channel: {}", channel);

        // Start the process
        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                info!("Started streamlink PID: {}", pid);

                self.current_process = Some(child);
                self.current_url = Some(url.to_string());
                Ok(())
            }
            Err(e) => {
                error!("Failed to start streamlink: {}", e);
                Err(format!("Failed to start stream: {}. Make sure streamlink is installed and in your PATH.", e))
            }
        }
    }

    pub fn stop_stream(&mut self) -> bool {
        if let Some(mut process) = self.current_process.take() {
            let pid = process.id();
            info!("Stopping stream PID: {}", pid);

            // First, try to kill any VLC processes that might be running
            self.kill_vlc_processes();

            // Kill the streamlink process
            if let Err(e) = process.kill() {
                warn!("Failed to kill process {}: {}", pid, e);
                return false; // Stop stream indicates failure
            }

            // Wait for it to die
            if let Err(e) = process.wait() {
                warn!("Process {} exited abnormally: {}", pid, e);
            }

            self.current_url = None;
            info!("Stream stopped successfully");
            true
        } else {
            info!("No active stream to stop");
            false
        }
    }

    // Helper function to kill VLC processes
    pub fn kill_vlc_processes(&self) {
        #[cfg(windows)]
        {
            match std::process::Command::new("taskkill")
                .args(["/F", "/IM", "vlc.exe"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
            {
                Ok(output) => {
                    if output.status.success() {
                        info!("Successfully killed VLC processes");
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stderr.is_empty() {
                            info!("No VLC processes were running to kill");
                        } else {
                            warn!("Failed to kill VLC processes: {}", stderr);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to execute taskkill command: {}", e);
                }
            }
        }

        #[cfg(not(windows))]
        {
            match std::process::Command::new("pkill").arg("vlc").output() {
                Ok(output) => {
                    if output.status.success() {
                        info!("Successfully killed VLC processes");
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stderr.contains("no process found") {
                            info!("No VLC processes were running to kill");
                        } else {
                            warn!("Failed to kill VLC processes: {}", stderr);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to execute pkill command: {}", e);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_current_url(&self) -> Option<&String> {
        self.current_url.as_ref()
    }

    #[allow(dead_code)]
    pub fn is_streaming(&self) -> bool {
        self.current_process.is_some()
    }

    // Optimized streamlink availability check with static caching
    fn is_streamlink_available(&self) -> bool {
        use std::sync::OnceLock;
        static STREAMLINK_AVAILABLE: OnceLock<bool> = OnceLock::new();

        *STREAMLINK_AVAILABLE.get_or_init(|| {
            // Check if streamlink is in PATH by trying to run --version
            let mut cmd = Command::new("streamlink");
            cmd.arg("--version");

            #[cfg(windows)]
            cmd.creation_flags(CREATE_NO_WINDOW);

            cmd.stdout(Stdio::null()).stderr(Stdio::null());

            match cmd.status() {
                Ok(status) => {
                    let available = status.success();
                    if available {
                        info!("Streamlink detected and available");
                    } else {
                        warn!("Streamlink command failed with status: {}", status);
                    }
                    available
                }
                Err(e) => {
                    warn!("Streamlink not found: {}", e);
                    false
                }
            }
        })
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        // Ensure we clean up any running processes when the manager is dropped
        if self.current_process.is_some() {
            info!("ProcessManager dropping, stopping active stream");
            self.stop_stream();
        }

        // Additional cleanup: kill any remaining VLC processes
        self.kill_vlc_processes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_manager_creation() {
        let manager = ProcessManager::new();
        assert!(!manager.is_streaming());
        assert!(manager.get_current_url().is_none());
    }

    #[test]
    fn test_multiple_stop_calls() {
        let mut manager = ProcessManager::new();

        // Stopping when no stream is running should return false
        assert!(!manager.stop_stream());
        assert!(false == manager.stop_stream());
    }
}
