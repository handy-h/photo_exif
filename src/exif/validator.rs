use crate::model::ExtensionMismatch;
use anyhow::Result;
use image::ImageFormat;
use std::path::Path;

/// EXIF 和文件格式验证器
pub struct ExifValidator;

impl ExifValidator {
    /// 检测图片实际格式（通过 magic bytes）
    pub fn detect_format(path: &Path) -> Result<Option<ImageFormat>> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        let format = image::guess_format(&mut reader)?;
        Ok(Some(format))
    }

    /// 检测文件扩展名是否与实际格式匹配
    pub fn check_extension(path: &Path) -> Result<Option<ExtensionMismatch>> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        let actual_format = match image::guess_format(&mut reader) {
            Ok(f) => f,
            Err(_) => return Ok(None),
        };

        let actual_ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        let expected_exts = actual_format.extensions_str();
        let expected_ext = expected_exts.into_iter().next().unwrap_or("bin");

        if actual_ext.is_empty() {
            return Ok(Some(ExtensionMismatch {
                actual_format: Self::format_name(actual_format),
                expected_ext: format!(".{}", expected_ext),
                actual_ext: "(无)".to_string(),
            }));
        }

        if actual_ext != expected_ext {
            return Ok(Some(ExtensionMismatch {
                actual_format: Self::format_name(actual_format),
                expected_ext: format!(".{}", expected_ext),
                actual_ext: format!(".{}", actual_ext),
            }));
        }

        Ok(None)
    }

    /// 获取格式的可读名称
    fn format_name(format: ImageFormat) -> String {
        match format {
            ImageFormat::Png => "PNG".into(),
            ImageFormat::Jpeg => "JPEG".into(),
            ImageFormat::Gif => "GIF".into(),
            ImageFormat::WebP => "WebP".into(),
            ImageFormat::Tiff => "TIFF".into(),
            ImageFormat::Bmp => "BMP".into(),
            ImageFormat::Dds => "DDS".into(),
            ImageFormat::Dng => "DNG".into(),
            ImageFormat::Hdr => "HDR".into(),
            ImageFormat::Ico => "ICO".into(),
            ImageFormat::OpenExr => "OpenEXR".into(),
            ImageFormat::Pnm => "PNM".into(),
            ImageFormat::Qoi => "QOI".into(),
            ImageFormat::Tga => "TGA".into(),
            _ => format!("{:?}", format),
        }
    }

    /// 修正文件扩展名
    pub fn fix_extension(path: &Path) -> Result<PathBuf> {
        let new_ext = Self::detect_format(path)?
            .and_then(|f| f.extensions_str().into_iter().next().map(|s| s.to_string()))
            .unwrap_or_else(|| "bin".to_string());

        let new_path = if let Some(stem) = path.file_stem() {
            path.with_file_name(format!("{}.{}", stem.to_string_lossy(), new_ext))
        } else {
            path.with_extension(new_ext)
        };

        std::fs::rename(path, &new_path)?;
        Ok(new_path)
    }
}
