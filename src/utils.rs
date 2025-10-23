//! Utility functions for the application

use std::path::Path;

use tokio::process::Command;

use crate::error::{BilidownError, Result};

pub async fn run_ffmpeg_command(args: &[&str]) -> Result<()> {
    let output = Command::new("ffmpeg")
        .args(args)
        .output()
        .await
        .map_err(|e| BilidownError::ConversionError(format!("ffmpeg执行失败: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BilidownError::ConversionError(format!(
            "ffmpeg命令失败: {}",
            stderr
        )));
    }

    Ok(())
}

pub fn sanitize_filename(filename: &str) -> String {
    filename
        .replace("/", "_")
        .replace(":", "_")
        .replace("<", "_")
        .replace(">", "_")
        .replace("|", "_")
        .replace("?", "_")
        .replace("*", "_")
        .replace("\"", "_")
        .replace("\\", "_")
}

pub fn validate_file_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(BilidownError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("文件不存在: {:?}", path),
        )));
    }
    Ok(())
}

pub fn validate_file_not_empty(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| BilidownError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    if metadata.len() == 0 {
        return Err(BilidownError::ConversionError(format!(
            "文件为空: {:?}",
            path
        )));
    }

    Ok(())
}
