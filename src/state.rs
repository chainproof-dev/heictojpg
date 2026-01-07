use std::sync::Arc;
use crate::config::Config;
use crate::worker::WorkerPool;

/// Application state shared across handlers
pub struct AppState {
    pub worker_pool: WorkerPool,
    pub config: Arc<Config>,
}
