use std::io::Cursor;

use jpeg_encoder::{ColorType as JpegColorType, Encoder as JpegEncoder};
use png::{BitDepth, ColorType, Encoder};
use screencapturekit::cg::CGRect;
use screencapturekit::screenshot_manager::SCScreenshotManager;

use crate::cli::ScreenshotImageFormat;
use crate::error::{AxError, Result};

#[derive(Debug, Clone)]
pub struct ScreenshotImage {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ScreenshotImageFormat,
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

pub fn ensure_screen_capture_permission(prompt: bool) -> Result<()> {
    let granted = unsafe {
        if CGPreflightScreenCaptureAccess() {
            true
        } else if prompt {
            CGRequestScreenCaptureAccess()
        } else {
            false
        }
    };

    if granted {
        Ok(())
    } else {
        Err(AxError::ScreenCaptureNotTrusted)
    }
}

pub fn capture_rect(rect: CGRect, format: ScreenshotImageFormat) -> Result<ScreenshotImage> {
    let image = SCScreenshotManager::capture_image_in_rect(rect)
        .map_err(|err| AxError::ScreenshotUnavailable(err.to_string()))?;
    let rgba = image
        .rgba_data()
        .map_err(|err| AxError::ScreenshotUnavailable(err.to_string()))?;

    let width = u32::try_from(image.width())
        .map_err(|_| AxError::ScreenshotUnavailable("image width exceeds u32".to_string()))?;
    let height = u32::try_from(image.height())
        .map_err(|_| AxError::ScreenshotUnavailable("image height exceeds u32".to_string()))?;
    let bytes = match format {
        ScreenshotImageFormat::Png => encode_png(width, height, &rgba)?,
        ScreenshotImageFormat::Jpeg => encode_jpeg(width, height, &rgba)?,
    };

    Ok(ScreenshotImage {
        bytes,
        width,
        height,
        format,
    })
}

fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::new());
    let mut encoder = Encoder::new(&mut cursor, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);

    let mut writer = encoder
        .write_header()
        .map_err(|err| AxError::ScreenshotUnavailable(err.to_string()))?;
    writer
        .write_image_data(rgba)
        .map_err(|err| AxError::ScreenshotUnavailable(err.to_string()))?;

    drop(writer);
    Ok(cursor.into_inner())
}

fn encode_jpeg(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>> {
    let jpeg_width = u16::try_from(width).map_err(|_| {
        AxError::ScreenshotUnavailable("image width exceeds jpeg limit".to_string())
    })?;
    let jpeg_height = u16::try_from(height).map_err(|_| {
        AxError::ScreenshotUnavailable("image height exceeds jpeg limit".to_string())
    })?;

    let mut rgb = Vec::with_capacity((width as usize) * (height as usize) * 3);
    for chunk in rgba.chunks_exact(4) {
        rgb.extend_from_slice(&chunk[..3]);
    }

    let mut output = Vec::new();
    let encoder = JpegEncoder::new(&mut output, 90);
    encoder
        .encode(&rgb, jpeg_width, jpeg_height, JpegColorType::Rgb)
        .map_err(|err| AxError::ScreenshotUnavailable(err.to_string()))?;
    Ok(output)
}
