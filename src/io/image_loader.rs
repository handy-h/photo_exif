use anyhow::{Context, Result};
use image::DynamicImage;
use std::path::Path;

/// 图片加载器 - 负责加载图片并转换为 egui 可用格式
pub struct ImageLoader;

impl ImageLoader {
    /// 加载图片并转换为 ColorImage
    pub fn to_color_image(img: &DynamicImage) -> egui::ColorImage {
        let rgba = img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba.into_raw();
        egui::ColorImage::from_rgba_unmultiplied(size, &pixels)
    }

    /// 加载图片用于预览
    pub fn load_for_preview(path: &Path) -> Result<(egui::ColorImage, [u32; 2])> {
        let img = image::open(path)
            .with_context(|| format!("无法加载图片: {}", path.display()))?;

        let size = [img.width(), img.height()];
        let color_img = Self::to_color_image(&img);

        Ok((color_img, size))
    }

    /// 加载图片并检测格式
    pub fn load_with_validation(
        path: &Path,
    ) -> Result<(egui::ColorImage, [u32; 2], Option<crate::model::ExtensionMismatch>)> {
        let (color_img, size) = Self::load_for_preview(path)?;

        let mismatch = crate::exif::ExifValidator::check_extension(path)?;

        Ok((color_img, size, mismatch))
    }

    /// 生成缩略图
    pub fn load_thumbnail(path: &Path, max_size: u32) -> Result<egui::ColorImage> {
        let img = image::open(path)
            .with_context(|| format!("无法加载图片: {}", path.display()))?;

        let thumb = img.thumbnail(max_size, max_size);
        Ok(Self::to_color_image(&thumb))
    }
}
