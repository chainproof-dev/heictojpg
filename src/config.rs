//! Configuration management
//!
//! Includes smart CPU detection for optimal resource utilization.

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

/// Smart CPU detection for optimal worker configuration
///
/// Strategy:
/// - Detects available parallelism (logical cores)
/// - For CPU-bound image processing, using all logical cores is beneficial
/// - Reserves 1 core for I/O tasks if we have more than 4 cores
/// - Ensures minimum of 2 workers and maximum based on available parallelism
fn detect_optimal_workers() -> usize {
    let logical_cores = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4);

    // Strategy for CPU-bound image conversion:
    // - Use most cores for conversion work
    // - Reserve 1 for async I/O if we have plenty (>4)
    // - Minimum 2 workers, maximum = logical cores
    let workers = if logical_cores > 4 {
        logical_cores - 1 // Reserve one for I/O
    } else {
        logical_cores // Use all cores on smaller systems
    };

    workers.max(2) // Ensure at least 2 workers
}

/// Calculate optimal queue size based on worker count
///
/// Strategy:
/// - Queue should be large enough to absorb burst traffic
/// - But not so large that we waste memory on pending requests
/// - Formula: workers * 4 for burst absorption, minimum 100
fn calculate_optimal_queue_size(worker_count: usize) -> usize {
    let base_queue = worker_count * 4;
    base_queue.max(100) // Minimum 100 for small worker counts
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok(); // Load .env if present

        // Detect optimal workers first (used for queue calculation if not overridden)
        let optimal_workers = detect_optimal_workers();

        let worker_count = env::var("WORKER_COUNT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(optimal_workers);

        // Queue size based on worker count for optimal throughput
        let default_queue = calculate_optimal_queue_size(worker_count);

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

            worker_count,

            queue_size: env::var("QUEUE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default_queue),

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
