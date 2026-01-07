//! Worker pool for CPU-bound image conversion

use crate::config::Config;
use crate::converter::{convert, ConvertOptions};
use crate::error::ConvertError;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Semaphore};

/// A conversion job
pub struct Job {
    pub input: Vec<u8>,
    pub quality: u8,
    pub response_tx: oneshot::Sender<Result<Vec<u8>, ConvertError>>,
}

/// Worker pool for handling conversion jobs
pub struct WorkerPool {
    job_tx: mpsc::Sender<Job>,
    semaphore: Arc<Semaphore>,
}

impl WorkerPool {
    /// Create a new worker pool
    pub fn new(config: &Config) -> Self {
        let (job_tx, mut job_rx) = mpsc::channel::<Job>(config.queue_size);
        let semaphore = Arc::new(Semaphore::new(config.worker_count));

        // Create options to share with workers
        let options = Arc::new(ConvertOptions {
            max_resolution: config.max_resolution,
            min_quality: config.min_quality,
            max_quality: config.max_quality,
        });

        // Spawn the job processor
        let sem = semaphore.clone();
        tokio::spawn(async move {
            while let Some(job) = job_rx.recv().await {
                // Backpressure fix: Acquire permit BEFORE spawning the task.
                // This ensures we only pull from the channel when we have capacity,
                // causing the channel to fill up and reject new requests when busy.
                let permit = match sem.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => break, // Semaphore closed
                };

                let opts = options.clone();

                // Spawn blocking task for CPU-bound work
                tokio::spawn(async move {
                    // Permit is held by this task and dropped when it completes
                    let _permit = permit;

                    // Run conversion in blocking thread pool
                    let result = tokio::task::spawn_blocking(move || {
                        convert(&job.input, job.quality, &opts)
                    })
                    .await
                    .unwrap_or_else(|e| Err(ConvertError::Internal(e.to_string())));

                    // Send result back (ignore if receiver dropped)
                    let _ = job.response_tx.send(result);
                });
            }
        });

        Self { job_tx, semaphore }
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

    /// Get current queue capacity
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}
