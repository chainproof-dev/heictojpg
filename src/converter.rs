//! Core HEIC to JPEG conversion engine

use crate::error::ConvertError;
use libheif_rs::{HeifContext, RgbChroma, ColorSpace, LibHeif};
use turbojpeg::{Compressor, Image, PixelFormat};

/// Decode HEIC bytes to RGB image buffer
fn decode_heic(data: &[u8], max_resolution: u32) -> Result<(Vec<u8>, u32, u32), ConvertError> {
    // Create LibHeif instance
    let lib_heif = LibHeif::new();
    
    // Create HEIF context from bytes
    let ctx = HeifContext::read_from_bytes(data)
        .map_err(|e| ConvertError::DecodeError(e.to_string()))?;

    // Get primary image handle
    let handle = ctx
        .primary_image_handle()
        .map_err(|e| ConvertError::DecodeError(e.to_string()))?;

    let width = handle.width();
    let height = handle.height();

    // Check resolution limits
    if width > max_resolution || height > max_resolution {
        return Err(ConvertError::ImageTooLarge {
            width,
            height,
            max: max_resolution,
        });
    }

    // Decode to RGB using LibHeif instance
    let image = lib_heif
        .decode(&handle, ColorSpace::Rgb(RgbChroma::Rgb), None)
        .map_err(|e| ConvertError::DecodeError(e.to_string()))?;

    let planes = image.planes();
    
    let interleaved = planes.interleaved
        .ok_or_else(|| ConvertError::DecodeError("Failed to get interleaved RGB data".to_string()))?;

    // Copy pixel data
    let stride = interleaved.stride;
    let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
    
    for y in 0..height as usize {
        let row_start = y * stride;
        let row_end = row_start + (width as usize * 3);
        rgb_data.extend_from_slice(&interleaved.data[row_start..row_end]);
    }

    Ok((rgb_data, width, height))
}

/// Encode RGB buffer to JPEG bytes
fn encode_jpeg(rgb_data: &[u8], width: u32, height: u32, quality: u8, min_q: u8, max_q: u8) -> Result<Vec<u8>, ConvertError> {
    // Validate quality
    let quality = quality.clamp(min_q, max_q);

    // Create compressor
    let mut compressor = Compressor::new()
        .map_err(|e| ConvertError::EncodeError(e.to_string()))?;

    compressor.set_quality(quality as i32);
    compressor.set_subsamp(turbojpeg::Subsamp::Sub2x2); // 4:2:0 chroma subsampling

    // Create image wrapper
    let image = Image {
        pixels: rgb_data,
        width: width as usize,
        pitch: width as usize * 3,
        height: height as usize,
        format: PixelFormat::RGB,
    };

    // Compress
    let jpeg_data = compressor
        .compress_to_vec(image)
        .map_err(|e| ConvertError::EncodeError(e.to_string()))?;

    Ok(jpeg_data)
}

/// Conversion options
pub struct ConvertOptions {
    pub max_resolution: u32,
    pub min_quality: u8,
    pub max_quality: u8,
}

/// Convert HEIC bytes to JPEG bytes
/// 
/// # Arguments
/// * `heic_data` - Raw HEIC file bytes
/// * `quality` - JPEG quality (60-95)
/// * `options` - Conversion limits and options
pub fn convert(heic_data: &[u8], quality: u8, options: &ConvertOptions) -> Result<Vec<u8>, ConvertError> {
    // Validate quality
    if quality < options.min_quality || quality > options.max_quality {
        return Err(ConvertError::InvalidQuality(quality));
    }

    // Decode HEIC to RGB
    let (rgb_data, width, height) = decode_heic(heic_data, options.max_resolution)?;

    // Encode RGB to JPEG
    let jpeg_data = encode_jpeg(&rgb_data, width, height, quality, options.min_quality, options.max_quality)?;

    Ok(jpeg_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_quality() {
        let options = ConvertOptions {
            max_resolution: 1000,
            min_quality: 60,
            max_quality: 95,
        };
        let result = convert(&[], 50, &options);
        assert!(matches!(result, Err(ConvertError::InvalidQuality(50))));
    }
}
