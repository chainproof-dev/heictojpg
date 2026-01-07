use crate::config::Config;
use crate::worker::WorkerPool;
use std::sync::Arc;

/// Application state shared across handlers
pub struct AppState {
    pub worker_pool: WorkerPool,
    pub config: Arc<Config>,
}
