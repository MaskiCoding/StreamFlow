#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// Core modules
mod process_manager;
mod settings;
mod streamlink;
mod twitch;
mod types;

// New modular structure
mod config;
mod error;
mod functionality;
mod util;
mod window;

use functionality::{cleanup_processes, AppState};
use util::logger;

fn main() {
    // Initialize configuration
    config::Config::init();

    // Initialize logging
    logger::init(false);

    // Panic hook for better error reporting
    std::panic::set_hook(Box::new(|info| {
        log::error!("Panic occurred: {:?}", info);
    }));

    log::info!("Starting StreamFlow-Tauri v{}", env!("CARGO_PKG_VERSION"));

    // Initialize application state
    let app_state = AppState::new();

    let context = tauri::generate_context!("tauri.conf.json");

    log::info!("StreamFlow-Tauri application starting...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_http::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Streaming commands
            functionality::start_stream,
            functionality::stop_stream,
            functionality::get_current_stream,
            functionality::is_vlc_running,
            functionality::validate_twitch_url,
            functionality::extract_channel_name,
            functionality::normalize_twitch_url,
            functionality::load_settings,
            functionality::save_settings,
            functionality::add_quick_stream,
            functionality::remove_quick_stream,
            functionality::check_stream_status,
            functionality::update_quick_stream_status,
            functionality::get_app_version,
            // Window management commands
            window::minimize,
            window::toggle_maximize,
            window::close,
            // Configuration commands
            config::get_config,
            config::set_config,
            config::read_config_file,
            config::write_config_file,
            config::default_config,
            // Utility commands
            util::helpers::get_platform,
        ])
        .setup(|app: &mut App| {
            // Initialize window
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.set_title("StreamFlow") {
                    log::warn!("Failed to set window title: {}", e);
                }
            }

            log::info!("Application setup completed successfully");
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    log::info!("Window close requested");

                    // Check if we should minimize to tray instead
                    if let Ok(config) = config::Config::load() {
                        if config.behavior.close_to_tray {
                            log::info!("Minimizing to tray instead of closing");
                            if let Err(e) = window.hide() {
                                log::error!("Failed to hide window: {}", e);
                            }
                            api.prevent_close();
                            return;
                        }
                    }

                    // Perform cleanup on actual close
                    log::info!("Performing application cleanup...");
                    if let Some(app_state) = window.app_handle().try_state::<AppState>() {
                        cleanup_processes(&app_state);
                    }
                }
                tauri::WindowEvent::Destroyed => {
                    log::info!("Window destroyed");
                }
                _ => {}
            }
        })
        .run(context)
        .expect("error while running tauri application");

    log::info!("StreamFlow-Tauri application exited");
}
