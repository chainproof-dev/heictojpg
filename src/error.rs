//! Error types for the HEIC to JPG converter

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("Failed to decode HEIC: {0}")]
    DecodeError(String),

    #[error("Failed to encode JPEG: {0}")]
    EncodeError(String),

    #[error("Invalid file: {0}")]
    ValidationError(String),

    #[error("File too large: {size} bytes (max: {max} bytes)")]
    FileTooLarge { size: usize, max: usize },

    #[error("Image too large: {width}x{height} (max: {max}x{max})")]
    ImageTooLarge { width: u32, height: u32, max: u32 },

    #[error("Invalid quality: {0} (must be 60-95)")]
    InvalidQuality(u8),

    #[error("Queue full, try again later")]
    QueueFull,

    #[error("Conversion timeout")]
    Timeout,

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ConvertError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ConvertError::DecodeError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ConvertError::EncodeError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ConvertError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ConvertError::FileTooLarge { .. } => (StatusCode::PAYLOAD_TOO_LARGE, self.to_string()),
            ConvertError::ImageTooLarge { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            ConvertError::InvalidQuality(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ConvertError::QueueFull => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            ConvertError::Timeout => (StatusCode::GATEWAY_TIMEOUT, self.to_string()),
            ConvertError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            ),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

// Add serde for JSON responses
use serde_json;
