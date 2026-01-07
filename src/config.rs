//! Configuration management
use dotenvy::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Maximum file size allowed in bytes
    pub max_file_size: usize,
    /// Maximum image resolution (width or height)
    pub max_resolution: u32,
    /// Default JPEG quality (1-100)
    pub default_quality: u8,
    /// Minimum allowed quality
    pub min_quality: u8,
    /// Maximum allowed quality
    pub max_quality: u8,
    /// Number of worker threads
    pub worker_count: usize,
    /// Maximum pending jobs in queue
    pub queue_size: usize,
    /// Server port
    pub server_port: u16,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Directory to store uploaded files for audit
    pub upload_dir: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok(); // Load .env if present

        Self {
            max_file_size: env::var("MAX_FILE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50 * 1024 * 1024), // 50MB

            max_resolution: env::var("MAX_RESOLUTION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(16384),

            default_quality: env::var("DEFAULT_QUALITY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(85),

            min_quality: env::var("MIN_QUALITY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),

            max_quality: env::var("MAX_QUALITY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(95),

            worker_count: env::var("WORKER_COUNT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| {
                    std::thread::available_parallelism()
                        .map(|p| p.get())
                        .unwrap_or(4)
                }),

            queue_size: env::var("QUEUE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),

            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),

            request_timeout_secs: env::var("REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),

            upload_dir: env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string()),
        }
    }
}

// Defaults for backward compatibility/testing if needed,
// though direct usage should prefer the struct.
