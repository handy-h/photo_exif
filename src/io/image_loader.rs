use crate::model::{AppState, ExtensionMismatch};
use anyhow::{Context, Result};
use image::{DynamicImage, ImageReader};
use std::path::Path;

/// 图片加载器 - 负责加载图片预览图和检测格式
pub struct ImageLoader;

impl ImageLoader {
    /// 加载图片为 DynamicImage
    pub fn load_image(path: &Path) -> Result<DynamicImage> {
        let img = ImageReader::open(path)?
            .with_guessed_format()?
            .decode()?;
        Ok(img)
    }

    /// 将 DynamicImage 转换为 egui::ColorImage（用于预览）
    pub fn to_color_image(img: &DynamicImage) -> egui::ColorImage {
        let rgba = img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba
            .pixels()
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        egui::ColorImage::from_rgba_unmultiplied(size, &pixels)
    }

    /// 加载图片并返回 (ColorImage, [width, height])
    pub fn load_for_preview(path: &Path) -> Result<(egui::ColorImage, [u32; 2])> {
        let img = Self::load_image(path)?;
        let color_img = Self::to_color_image(&img);
        let size = [img.width(), img.height()];
        Ok((color_img, size))
    }

    /// 检查扩展名是否匹配，返回 (图片数据, 校验结果)
    pub fn load_with_validation(path: &Path) -> Result<(egui::ColorImage, [u32; 2], Option<ExtensionMismatch>)> {
        let mismatch = match ExifValidator::check_extension(path) {
            Ok(m) => m,
            Err(_) => None,
        };

        let (color_img, size) = Self::load_for_preview(path)?;
        Ok((color_img, size, mismatch))
    }
}
