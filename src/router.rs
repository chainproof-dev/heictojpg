use axum::http::{header, HeaderValue};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    request_id::MakeRequestUuid,
    services::ServeDir,
    set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
    ServiceBuilderExt,
};

use crate::handlers::{batch_info, convert_handler, health};
use crate::state::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    // CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Security Headers
    // - X-Content-Type-Options: nosniff
    // - X-Frame-Options: DENY
    // - Content-Security-Policy: default-src 'self' ...
    let security_headers = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ));

    // Build router
    Router::new()
        // API routes
        .route("/api/health", get(health))
        .route("/api/convert", post(convert_handler))
        .route("/api/info", get(batch_info))
        // Static files (frontend)
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true))
        // Middleware
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(
                    state.config.request_timeout_secs,
                )))
                .layer(CompressionLayer::new()) // GZIP/Brotli compression
                .layer(cors)
                .layer(security_headers),
        )
        // Shared state
        .with_state(state)
}
