//! QAIVRA Phase 7 - Video engine: upload, probe, and frame extraction.
//! Requires FFmpeg + FFprobe as external local prerequisites.

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use sha2::{Digest, Sha256};

pub const MAX_VIDEO_SIZE: u64 = 1_073_741_824;
pub const MIN_INTERVAL_SECS: f64 = 1.0;
pub const MAX_INTERVAL_SECS: f64 = 30.0;
pub const MAX_FRAMES_PER_EXTRACTION: u32 = 10_000;

pub const SUPPORTED_VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "avi", "mkv", "webm"];
pub const SUPPORTED_VIDEO_MIME_TYPES: &[&str] = &[
    "video/mp4", "video/quicktime", "video/x-msvideo",
    "video/x-matroska", "video/webm",
];

#[derive(Debug, Serialize, Clone)]
pub struct VideoProbeResult {
    pub duration_secs: f64,
    pub width: i32,
    pub height: i32,
    pub frame_rate: f64,
    pub media_type: String,
}

/// Validate video file path: extension, existence, regular file, size, readability.
pub fn validate_video_path(source: &Path) -> Result<PathBuf, String> {
    let canonical = source.canonicalize()
        .map_err(|e| format!("Cannot access video file: {}", e))?;
    if !canonical.is_file() {
        return Err("Video path is not a regular file".to_string());
    }
    let ext = canonical.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if !SUPPORTED_VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        return Err(format!("Unsupported video format '.{}'. Supported: {}", ext,
            SUPPORTED_VIDEO_EXTENSIONS.join(", ")));
    }
    let metadata = std::fs::metadata(&canonical)
        .map_err(|e| format!("Cannot read video metadata: {}", e))?;
    if metadata.len() > MAX_VIDEO_SIZE {
        return Err(format!("Video file too large ({} MiB, max {} MiB)",
            metadata.len() / 1_048_576, MAX_VIDEO_SIZE / 1_048_576));
    }
    Ok(canonical)
}

/// Probe video using FFprobe. Returns structured metadata.
pub fn probe_video(video_path: &Path) -> Result<VideoProbeResult, String> {
    let output = Command::new("ffprobe")
        .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams",
            video_path.to_str().ok_or("Invalid video path")?])
        .output()
        .map_err(|e| format!("FFprobe not found: {}. Install FFmpeg.", e))?;
    if !output.status.success() {
        return Err(format!("FFprobe failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse FFprobe output: {}", e))?;

    let mut duration_secs = 0.0_f64;
    if let Some(d) = parsed["format"]["duration"].as_str() {
        duration_secs = d.parse::<f64>().unwrap_or(0.0);
    }
    let mut width = 0_i32; let mut height = 0_i32; let mut frame_rate = 0.0_f64;
    let mut media_type = String::new();
    if let Some(streams) = parsed["streams"].as_array() {
        for s in streams {
            if s["codec_type"].as_str() == Some("video") {
                width = s["width"].as_i64().unwrap_or(0) as i32;
                height = s["height"].as_i64().unwrap_or(0) as i32;
                if let Some(fr_str) = s["r_frame_rate"].as_str() {
                    let parts: Vec<&str> = fr_str.split('/').collect();
                    if parts.len() == 2 {
                        let num = parts[0].parse::<f64>().unwrap_or(0.0);
                        let den = parts[1].parse::<f64>().unwrap_or(1.0);
                        if den > 0.0 { frame_rate = num / den; }
                    }
                }
                media_type = s["codec_name"].as_str()
                    .map(|c| format!("video/{}", c)).unwrap_or_else(|| "video/unknown".to_string());
                break;
            }
        }
    }
    Ok(VideoProbeResult { duration_secs, width, height, frame_rate, media_type })
}

/// SHA-256 hex digest of a file.
pub fn hash_file(path: &Path) -> Result<String, String> {
    let data = std::fs::read(path).map_err(|e| format!("Cannot read file for hashing: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Validate extraction interval bounds.
pub fn validate_interval(interval_secs: f64) -> Result<f64, String> {
    if interval_secs < MIN_INTERVAL_SECS || interval_secs > MAX_INTERVAL_SECS {
        return Err(format!("Interval must be between {}s and {}s", MIN_INTERVAL_SECS, MAX_INTERVAL_SECS));
    }
    if !interval_secs.is_finite() { return Err("Interval must be finite".to_string()); }
    Ok(interval_secs)
}

/// Compute frame count from duration and interval.
pub fn compute_frame_count(duration_secs: f64, interval_secs: f64) -> u32 {
    let count = (duration_secs / interval_secs).floor() as u32;
    count.min(MAX_FRAMES_PER_EXTRACTION)
}

/// Format milliseconds as HH:MM:SS.mmm
pub fn format_timestamp(ms: i64) -> String {
    let sign = if ms < 0 { "-" } else { "" };
    let abs_ms = ms.abs() as u64;
    let h = abs_ms / 3_600_000; let m = (abs_ms % 3_600_000) / 60_000;
    let s = (abs_ms % 60_000) / 1_000; let ms_part = abs_ms % 1_000;
    format!("{}{:02}:{:02}:{:02}.{:03}", sign, h, m, s, ms_part)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_interval_bounds() {
        assert!(validate_interval(0.5).is_err());
        assert!(validate_interval(1.0).is_ok());
        assert!(validate_interval(5.0).is_ok());
        assert!(validate_interval(30.0).is_ok());
        assert!(validate_interval(31.0).is_err());
    }

    #[test]
    fn test_validate_interval_finite() {
        assert!(validate_interval(f64::NAN).is_err());
        assert!(validate_interval(f64::INFINITY).is_err());
    }

    #[test]
    fn test_compute_frame_count() {
        assert_eq!(compute_frame_count(10.0, 1.0), 10);
        assert_eq!(compute_frame_count(10.0, 2.0), 5);
        assert_eq!(compute_frame_count(10.5, 2.0), 5);
        assert_eq!(compute_frame_count(0.5, 1.0), 0);
    }

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0), "00:00:00.000");
        assert_eq!(format_timestamp(1000), "00:00:01.000");
        assert_eq!(format_timestamp(61000), "00:01:01.000");
        assert_eq!(format_timestamp(3661000), "01:01:01.000");
    }

    #[test]
    fn test_supported_extensions_list() {
        assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"mp4"));
        assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"mov"));
        assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"avi"));
        assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"mkv"));
        assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"webm"));
    }
}
