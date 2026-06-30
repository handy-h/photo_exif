use crate::model::ExtensionMismatch;
use anyhow::Result;
use image::ImageFormat;
use std::path::Path;

/// EXIF 和文件格式验证器
pub struct ExifValidator;

impl ExifValidator {
    /// 检测图片实际格式（通过 magic bytes）
    pub fn detect_format(path: &Path) -> Result<Option<ImageFormat>> {
        let header = std::fs::read(path)?;
        if header.len() < 2 {
            return Ok(None);
        }
        let format = image::guess_format(&header)?;
        Ok(Some(format))
    }

    /// 检测文件扩展名是否与实际格式匹配
    pub fn check_extension(path: &Path) -> Result<Option<ExtensionMismatch>> {
        let header = std::fs::read(path)?;
        let actual_format = match image::guess_format(&header) {
            Ok(f) => f,
            Err(_) => return Ok(None),
        };

        let actual_ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        let expected_exts = actual_format.extensions_str();

        if actual_ext.is_empty() {
            return Ok(Some(ExtensionMismatch {
                actual_format: Self::format_name(actual_format),
                extension: "(无)".to_string(),
            }));
        }

        if !expected_exts.contains(&actual_ext.as_str()) {
            return Ok(Some(ExtensionMismatch {
                actual_format: Self::format_name(actual_format),
                extension: format!(".{}", actual_ext),
            }));
        }

        Ok(None)
    }

    /// 获取格式的可读名称
    fn format_name(format: ImageFormat) -> String {
        match format {
            ImageFormat::Png => "PNG".into(),
            ImageFormat::Jpeg => "JPEG".into(),
            ImageFormat::WebP => "WebP".into(),
            ImageFormat::Tiff => "TIFF".into(),
            ImageFormat::Bmp => "BMP".into(),
            _ => "Unknown".into(),
        }
    }
}
