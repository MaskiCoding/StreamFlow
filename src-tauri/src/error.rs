use std::fmt;

/// Custom error types for StreamFlow-Tauri
#[derive(Debug)]
pub enum StreamFlowError {
    /// Stream-related errors
    StreamError(String),
    /// Configuration errors
    ConfigError(String),
    /// Process management errors
    ProcessError(String),
    /// Network/HTTP errors
    NetworkError(String),
    /// Validation errors
    ValidationError(String),
    /// File system errors
    FileSystemError(String),
    /// Generic errors
    GenericError(String),
}

impl fmt::Display for StreamFlowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StreamFlowError::StreamError(msg) => write!(f, "Stream error: {}", msg),
            StreamFlowError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            StreamFlowError::ProcessError(msg) => write!(f, "Process error: {}", msg),
            StreamFlowError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            StreamFlowError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            StreamFlowError::FileSystemError(msg) => write!(f, "File system error: {}", msg),
            StreamFlowError::GenericError(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for StreamFlowError {}

impl From<std::io::Error> for StreamFlowError {
    fn from(err: std::io::Error) -> Self {
        StreamFlowError::FileSystemError(err.to_string())
    }
}

impl From<reqwest::Error> for StreamFlowError {
    fn from(err: reqwest::Error) -> Self {
        StreamFlowError::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for StreamFlowError {
    fn from(err: serde_json::Error) -> Self {
        StreamFlowError::ConfigError(err.to_string())
    }
}

impl From<tauri::Error> for StreamFlowError {
    fn from(err: tauri::Error) -> Self {
        StreamFlowError::GenericError(err.to_string())
    }
}

impl From<StreamFlowError> for String {
    fn from(err: StreamFlowError) -> String {
        err.to_string()
    }
}

// Type alias for Results
pub type StreamFlowResult<T> = Result<T, StreamFlowError>;
