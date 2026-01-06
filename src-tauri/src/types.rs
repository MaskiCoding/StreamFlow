//! Shared type definitions for StreamFlow-Tauri
//!
//! This module contains common types used across the application
//! to avoid duplication between config.rs and settings.rs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Stream status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum StreamStatus {
    Online,
    Offline,
    #[default]
    Unknown,
}

/// Saved stream structure for quick access streams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedStream {
    pub name: String,
    pub url: String,
    pub status: StreamStatus,
    pub last_checked: Option<DateTime<Utc>>,
}
