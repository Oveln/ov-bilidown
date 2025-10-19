//! A Bilibili audio downloader library

// Re-export key types for convenience
pub use crate::error::{BilidownError, Result};

pub mod api;
pub mod config;
pub mod converter;
pub mod download;
pub mod error;
pub mod models;
pub mod user;
pub mod utils;
pub mod video;
pub mod wbi;

// Re-export commonly used types
pub use models::VideoBasicInfo;