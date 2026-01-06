use crate::error::{StreamFlowError, StreamFlowResult};
use tauri::{AppHandle, Manager, WebviewWindow};

/// Window management utilities for StreamFlow-Tauri
pub struct WindowManager;

impl WindowManager {
    /// Get the main window from the app handle
    pub fn get_main_window(app: &AppHandle) -> StreamFlowResult<WebviewWindow> {
        app.get_webview_window("main")
            .ok_or_else(|| StreamFlowError::GenericError("Main window not found".to_string()))
    }

    /// Set up window properties
    pub fn setup_window(window: &WebviewWindow) -> StreamFlowResult<()> {
        window.set_title("StreamFlow")?;
        log::info!("Window setup completed");
        Ok(())
    }

    /// Show the window and bring it to front
    pub fn show_window(window: &WebviewWindow) -> StreamFlowResult<()> {
        window.show()?;
        window.set_focus()?;
        Ok(())
    }

    /// Hide the window
    pub fn hide_window(window: &WebviewWindow) -> StreamFlowResult<()> {
        window.hide()?;
        Ok(())
    }

    /// Minimize the window
    pub fn minimize_window(window: &WebviewWindow) -> StreamFlowResult<()> {
        window.minimize()?;
        Ok(())
    }

    /// Toggle window maximization
    pub fn toggle_maximize(window: &WebviewWindow) -> StreamFlowResult<()> {
        if window.is_maximized()? {
            window.unmaximize()?;
        } else {
            window.maximize()?;
        }
        Ok(())
    }

    /// Close the window
    pub fn close_window(window: &WebviewWindow) -> StreamFlowResult<()> {
        window.close()?;
        Ok(())
    }

    /// Get window size
    pub fn get_window_size(window: &WebviewWindow) -> StreamFlowResult<(f64, f64)> {
        let size = window.inner_size()?;
        Ok((size.width as f64, size.height as f64))
    }

    /// Set window size
    pub fn set_window_size(window: &WebviewWindow, width: f64, height: f64) -> StreamFlowResult<()> {
        window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: width as u32,
            height: height as u32,
        }))?;
        Ok(())
    }

    /// Center the window on screen
    pub fn center_window(window: &WebviewWindow) -> StreamFlowResult<()> {
        window.center()?;
        Ok(())
    }

    /// Make window always on top
    pub fn set_always_on_top(window: &WebviewWindow, always_on_top: bool) -> StreamFlowResult<()> {
        window.set_always_on_top(always_on_top)?;
        Ok(())
    }

    /// Set window decorations
    pub fn set_decorations(window: &WebviewWindow, decorations: bool) -> StreamFlowResult<()> {
        window.set_decorations(decorations)?;
        Ok(())
    }

    /// Set window resizable
    pub fn set_resizable(window: &WebviewWindow, resizable: bool) -> StreamFlowResult<()> {
        window.set_resizable(resizable)?;
        Ok(())
    }

    /// Set window minimizable
    pub fn set_minimizable(window: &WebviewWindow, minimizable: bool) -> StreamFlowResult<()> {
        window.set_minimizable(minimizable)?;
        Ok(())
    }

    /// Set window maximizable
    pub fn set_maximizable(window: &WebviewWindow, maximizable: bool) -> StreamFlowResult<()> {
        window.set_maximizable(maximizable)?;
        Ok(())
    }

    /// Set window closable
    pub fn set_closable(window: &WebviewWindow, closable: bool) -> StreamFlowResult<()> {
        window.set_closable(closable)?;
        Ok(())
    }
}

// Tauri commands for window management
#[tauri::command]
pub async fn minimize(app: AppHandle) -> Result<String, String> {
    match app.get_webview_window("main") {
        Some(window) => match window.minimize() {
            Ok(_) => Ok("Window minimized".to_string()),
            Err(e) => Err(format!("Failed to minimize window: {}", e)),
        },
        None => Err("Main window not found".to_string()),
    }
}

#[tauri::command]
pub async fn toggle_maximize(app: AppHandle) -> Result<String, String> {
    match app.get_webview_window("main") {
        Some(window) => match window.is_maximized() {
            Ok(true) => match window.unmaximize() {
                Ok(_) => Ok("Window restored".to_string()),
                Err(e) => Err(format!("Failed to restore window: {}", e)),
            },
            Ok(false) => match window.maximize() {
                Ok(_) => Ok("Window maximized".to_string()),
                Err(e) => Err(format!("Failed to maximize window: {}", e)),
            },
            Err(e) => Err(format!("Failed to check window state: {}", e)),
        },
        None => Err("Main window not found".to_string()),
    }
}

#[tauri::command]
pub async fn close(app: AppHandle) -> Result<String, String> {
    match app.get_webview_window("main") {
        Some(window) => match window.close() {
            Ok(_) => Ok("Window closed".to_string()),
            Err(e) => Err(format!("Failed to close window: {}", e)),
        },
        None => Err("Main window not found".to_string()),
    }
}
