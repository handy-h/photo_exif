use crate::model::{AppState, ExifTag, ExifValue};
use crate::exif::{ExifReader, ExifWriter, ExifValidator};
use anyhow::{Context, Result};
use std::path::Path;

/// 文件操作 - 负责打开、保存、重置等文件相关操作
pub struct FileOps;

impl FileOps {
    /// 打开单个图片文件
    pub fn open_file(app: &mut AppState, path: std::path::PathBuf) -> Result<()> {
        // 加载图片预览
        let (color_img, size) = ImageLoader::load_for_preview(&path)
            .with_context(|| format!("无法加载图片: {}", path.display()))?;

        // 读取 EXIF
        let (exif_entries, format) = ExifReader::read(&path)
            .unwrap_or_default();

        // 更新状态
        app.current_image = Some(color_img);
        app.image_size = Some(size);
        app.exif_entries = exif_entries;
        app.original_exif = app.exif_entries.clone();
        app.undo_stack.clear();
        app.search_query.clear();
        app.selected_tags.clear();
        app.editing_tag = None;

        // 重置分组展开状态
        app.expanded_groups.insert(crate::model::ExifGroup::CameraInfo, true);
        app.expanded_groups.insert(crate::model::ExifGroup::Exposure, true);
        app.expanded_groups.insert(crate::model::ExifGroup::GPS, true);

        // 检查扩展名
        if let Ok(Some(mismatch)) = ExifValidator::check_extension(&path) {
            app.extension_warning = Some(mismatch);
        } else {
            app.extension_warning = None;
        }

        app.zoom = 1.0;
        app.clear_status();
        app.set_status(format!("已打开: {}", path.display()), crate::model::StatusLevel::Success);

        Ok(())
    }

    /// 打开文件夹
    pub fn open_folder(app: &mut AppState, path: std::path::PathBuf) -> Result<()> {
        let extensions = ["jpg", "jpeg", "png", "webp", "tiff", "tif", "bmp"];

        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&path)
            .with_context(|| format!("无法读取文件夹: {}", path.display()))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| extensions.contains(&e.to_lowercase().as_str()))
                    .unwrap_or(false)
            })
            .collect();

        files.sort();

        if files.is_empty() {
            app.set_status("文件夹中没有支持的图片文件", crate::model::StatusLevel::Warning);
            return Ok(());
        }

        app.folder_path = Some(path.clone());
        app.file_paths = files;
        app.current_index = 0;

        // 加载第一张
        if let Some(first) = app.file_paths.first() {
            Self::open_file(app, first.clone())?;
        }

        Ok(())
    }

    /// 切换到上一张
    pub fn prev_image(app: &mut AppState) -> Result<()> {
        if app.file_paths.is_empty() {
            return Ok(());
        }

        if app.has_unsaved_changes() {
            app.set_status("有未保存的更改，请先保存或重置", crate::model::StatusLevel::Warning);
            return Ok(());
        }

        if app.current_index > 0 {
            app.current_index -= 1;
            if let Some(path) = app.current_path().cloned() {
                Self::open_file(app, path)?;
            }
        }

        Ok(())
    }

    /// 切换到下一张
    pub fn next_image(app: &mut AppState) -> Result<()> {
        if app.file_paths.is_empty() {
            return Ok(());
        }

        if app.has_unsaved_changes() {
            app.set_status("有未保存的更改，请先保存或重置", crate::model::StatusLevel::Warning);
            return Ok(());
        }

        if app.current_index + 1 < app.file_paths.len() {
            app.current_index += 1;
            if let Some(path) = app.current_path().cloned() {
                Self::open_file(app, path)?;
            }
        }

        Ok(())
    }

    /// 保存当前图片的 EXIF
    pub fn save_exif(app: &mut AppState) -> Result<()> {
        if let Some(path) = app.current_path() {
            // 保存原始 EXIF 快照（用于重置）
            let original = app.original_exif.clone();
            let modified = app.exif_entries.clone();

            // 找出变更的字段
            let changes: Vec<_> = modified
                .iter()
                .filter(|(k, v)| original.get(k) != Some(v))
                .collect();

            if changes.is_empty() {
                app.set_status("没有需要保存的更改", crate::model::StatusLevel::Info);
                return Ok(());
            }

            // 写入 EXIF
            ExifWriter::write(path, &modified)?;

            app.original_exif = modified;
            app.undo_stack.clear();
            app.set_status(
                format!("已保存 {} 个字段的修改", changes.len()),
                crate::model::StatusLevel::Success,
            );
        }

        Ok(())
    }

    /// 重置为原始 EXIF
    pub fn reset_exif(app: &mut AppState) {
        if app.has_unsaved_changes() {
            app.exif_entries = app.original_exif.clone();
            app.undo_stack.clear();
            app.set_status("已重置为原始 EXIF", crate::model::StatusLevel::Info);
        }
    }

    /// 修正文件扩展名
    pub fn fix_extension(app: &mut AppState) -> Result<()> {
        if let Some(path) = app.current_path().cloned() {
            let new_path = ExifValidator::fix_extension(&path)?;

            // 更新文件列表
            if let Some(idx) = app.file_paths.iter().position(|p| p == &path) {
                app.file_paths[idx] = new_path.clone();
            }

            // 重新加载
            Self::open_file(app, new_path)?;
            app.extension_warning = None;
            app.set_status("扩展名已修正", crate::model::StatusLevel::Success);
        }

        Ok(())
    }

    /// 删除选中的 EXIF 字段
    pub fn delete_selected(app: &mut AppState) {
        for tag in app.selected_tags.drain(..) {
            if let Some(old_value) = app.exif_entries.remove(&tag) {
                app.push_undo(tag, old_value, ExifValue::Ascii("".into()));
            }
        }
        app.set_status("已删除选中的字段", crate::model::StatusLevel::Success);
    }
}
