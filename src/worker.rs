//! Worker pool for CPU-bound image conversion
//!
//! Uses a dedicated Rayon thread pool for optimal CPU-bound task scheduling.

use crate::config::Config;
use crate::converter::{convert, ConvertOptions};
use crate::error::ConvertError;
use rayon::ThreadPoolBuilder;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

/// A conversion job
pub struct Job {
    pub input: Vec<u8>,
    pub quality: u8,
    pub response_tx: oneshot::Sender<Result<Vec<u8>, ConvertError>>,
}

/// Worker pool backed by a dedicated Rayon thread pool
pub struct WorkerPool {
    job_tx: mpsc::Sender<Job>,
}

impl WorkerPool {
    /// Create a new worker pool with dedicated Rayon threads
    pub fn new(config: &Config) -> Self {
        let (job_tx, mut job_rx) = mpsc::channel::<Job>(config.queue_size);

        // Create conversion options to share with workers
        let options = Arc::new(ConvertOptions {
            max_resolution: config.max_resolution,
            min_quality: config.min_quality,
            max_quality: config.max_quality,
        });

        // Build a dedicated Rayon thread pool for CPU-bound work
        let rayon_pool = Arc::new(
            ThreadPoolBuilder::new()
                .num_threads(config.worker_count)
                .thread_name(|i| format!("converter-{}", i))
                .build()
                .expect("Failed to create Rayon thread pool"),
        );

        // Spawn the async job dispatcher
        tokio::spawn(async move {
            while let Some(job) = job_rx.recv().await {
                let opts = options.clone();
                let pool = rayon_pool.clone();

                // Spawn a Tokio task to bridge async -> sync
                tokio::task::spawn_blocking(move || {
                    // Execute on the dedicated Rayon pool
                    pool.install(|| {
                        let result = convert(&job.input, job.quality, &opts);
                        let _ = job.response_tx.send(result);
                    });
                });
            }
        });

        Self { job_tx }
    }

    /// Submit a job for conversion
    ///
    /// # Arguments
    /// * `input` - HEIC file bytes
    /// * `quality` - JPEG quality (60-95)
    ///
    /// # Returns
    /// * `Ok(oneshot::Receiver)` - Receiver for the result
    /// * `Err(ConvertError::QueueFull)` - Queue is full
    pub async fn submit(
        &self,
        input: Vec<u8>,
        quality: u8,
    ) -> Result<oneshot::Receiver<Result<Vec<u8>, ConvertError>>, ConvertError> {
        let (response_tx, response_rx) = oneshot::channel();

        let job = Job {
            input,
            quality,
            response_tx,
        };

        self.job_tx
            .try_send(job)
            .map_err(|_| ConvertError::QueueFull)?;

        Ok(response_rx)
    }
}
