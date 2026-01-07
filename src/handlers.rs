//! HTTP handlers for the HEIC to JPG converter API

use crate::error::ConvertError;
use crate::state::AppState;
use axum::{
    extract::{Multipart, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::{error, info, instrument};
use uuid::Uuid;

/// Health check endpoint
pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "heictojpg"
    }))
}

/// Convert HEIC to JPG endpoint
///
/// Accepts multipart form data with:
/// - `file`: HEIC file (required)
/// - `quality`: JPEG quality 60-95 (optional, default 85)
#[instrument(skip(state, multipart))]
pub async fn convert_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Response, ConvertError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut quality: u8 = state.config.default_quality;

    // Parse multipart form
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ConvertError::ValidationError(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| ConvertError::ValidationError(e.to_string()))?;

                // Check file size
                if data.len() > state.config.max_file_size {
                    return Err(ConvertError::FileTooLarge {
                        size: data.len(),
                        max: state.config.max_file_size,
                    });
                }

                file_data = Some(data.to_vec());
            }
            "quality" => {
                let q_str = field
                    .text()
                    .await
                    .map_err(|e| ConvertError::ValidationError(e.to_string()))?;

                quality = q_str.parse::<u8>().map_err(|_| {
                    ConvertError::ValidationError("Invalid quality value".to_string())
                })?;

                // Validate quality range
                if quality < state.config.min_quality || quality > state.config.max_quality {
                    return Err(ConvertError::InvalidQuality(quality));
                }
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    // Ensure we have a file
    let file_data = file_data
        .ok_or_else(|| ConvertError::ValidationError("Missing 'file' field".to_string()))?;

    info!(
        file_name = ?file_name,
        size = file_data.len(),
        quality = quality,
        "Processing conversion request"
    );

    // Securely save the file for audit
    // Format: YYYYMMDD-HHMMSS_UUID_original.heic
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let safe_id = Uuid::new_v4();
    let safe_filename = format!(
        "{}_{}_{}",
        timestamp,
        safe_id,
        file_name
            .clone()
            .unwrap_or_else(|| "unknown.heic".to_string())
            .replace(|c: char| !c.is_alphanumeric() && c != '.', "_") // Sanitize original name
    );

    let upload_path = std::path::Path::new(&state.config.upload_dir).join(&safe_filename);

    if let Err(e) = tokio::fs::write(&upload_path, &file_data).await {
        error!(error = %e, path = ?upload_path, "Failed to save uploaded file for audit");
        // We choose NOT to fail the request if audit save fails, but you could if strict audit is required.
    } else {
        info!(path = ?upload_path, "File saved for audit");
    }

    // Submit to worker pool
    let result_rx = state.worker_pool.submit(file_data, quality).await?;

    // Wait for result
    let jpeg_data = result_rx
        .await
        .map_err(|_| ConvertError::Internal("Worker dropped".to_string()))??;

    // Generate output filename
    // User requested "just numbers". Using millisecond timestamp ensures numeric, unique, and ordered.
    let output_name = format!("{}.jpg", Utc::now().timestamp_millis());

    info!(output_name = %output_name, size = jpeg_data.len(), "Conversion complete");

    // Build response with correct headers
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/jpeg"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", output_name),
            ),
        ],
        jpeg_data,
    )
        .into_response())
}

/// Batch convert endpoint info
pub async fn batch_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "endpoint": "/api/convert",
        "method": "POST",
        "description": "Convert HEIC to JPG",
        "fields": {
            "file": "HEIC file (required)",
            "quality": format!("JPEG quality {}-{} (optional, default {})",
                state.config.min_quality,
                state.config.max_quality,
                state.config.default_quality)
        },
        "limits": {
            "max_file_size": format!("{}MB", state.config.max_file_size / 1024 / 1024),
            "max_resolution": format!("{}x{}", state.config.max_resolution, state.config.max_resolution)
        }
    }))
}
