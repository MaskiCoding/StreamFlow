use std::env;
use std::path::Path;

/// Get the platform name as a string
pub fn get_platform() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "unknown"
    }
}

/// Check if running on Windows 7 or earlier
#[cfg(target_os = "windows")]
pub fn is_windows_7() -> bool {
    use windows_version::OsVersion;

    match OsVersion::current() {
        OsVersion::Windows(win) => {
            // Windows 7 is version 6.1
            win.major == 6 && win.minor == 1
        }
        _ => false,
    }
}

#[cfg(not(target_os = "windows"))]
pub fn is_windows_7() -> bool {
    false
}

/// Get the application data directory path
pub fn get_app_data_dir() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            Path::new(&appdata).join("StreamFlow-Tauri")
        } else {
            Path::new(".").join("data")
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            Path::new(&home).join("Library").join("Application Support").join("StreamFlow-Tauri")
        } else {
            Path::new(".").join("data")
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
            Path::new(&xdg_data_home).join("streamflow-tauri")
        } else if let Ok(home) = env::var("HOME") {
            Path::new(&home).join(".local").join("share").join("streamflow-tauri")
        } else {
            Path::new(".").join("data")
        }
    }
}

/// Get the configuration directory path
pub fn get_config_dir() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            Path::new(&appdata).join("StreamFlow-Tauri")
        } else {
            Path::new(".").join("config")
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            Path::new(&home).join("Library").join("Preferences").join("StreamFlow-Tauri")
        } else {
            Path::new(".").join("config")
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
            Path::new(&xdg_config_home).join("streamflow-tauri")
        } else if let Ok(home) = env::var("HOME") {
            Path::new(&home).join(".config").join("streamflow-tauri")
        } else {
            Path::new(".").join("config")
        }
    }
}

/// Check if the application is running in portable mode
pub fn is_portable() -> bool {
    // Check for portable.txt file in the executable directory
    let portable_marker = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("portable.txt")))
        .map(|p| p.exists())
        .unwrap_or(false);

    portable_marker
}