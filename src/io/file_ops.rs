use crate::model::{AppState, ExifTag, ExifValue, ThumbnailInfo};
use crate::exif::{ExifReader, ExifWriter, ExifValidator};
use anyhow::{Context, Result};

/// 文件操作 - 负责打开、保存、重置等文件相关操作
pub struct FileOps;

impl FileOps {
    /// 打开单个图片文件
    pub fn open_file(app: &mut AppState, path: std::path::PathBuf) -> Result<()> {
        // 加载图片预览
        let (color_img, size) = crate::io::ImageLoader::load_for_preview(&path)
            .with_context(|| format!("无法加载图片: {}", path.display()))?;

        // 读取 EXIF
        let (exif_entries, _format) = ExifReader::read(&path).unwrap_or_default();

        // 更新状态
        app.current_image = Some(color_img);
        app.image_size = Some(size);
        app.exif_entries = exif_entries.clone();
        app.original_exif = exif_entries;
        app.undo_stack.clear();
        app.search_query.clear();
        app.selected_tags.clear();
        app.editing_tag = None;
        app.pixel_perfect = false;

        // 重置分组展开状态
        app.expanded_groups
            .insert(crate::model::ExifGroup::CameraInfo, true);
        app.expanded_groups
            .insert(crate::model::ExifGroup::Exposure, true);
        app.expanded_groups
            .insert(crate::model::ExifGroup::GPS, true);

        // 检查扩展名
        if let Ok(Some(mismatch)) = ExifValidator::check_extension(&path) {
            app.extension_warning = Some(mismatch);
        } else {
            app.extension_warning = None;
        }

        // 加载缩略图
        Self::load_thumbnails_for_folder(app);
        app.zoom = 1.0;
        app.clear_status();
        app.set_status(
            format!("已打开: {}", path.display()),
            crate::model::StatusLevel::Success,
        );

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
            app.set_status(
                "文件夹中没有支持的图片文件",
                crate::model::StatusLevel::Warning,
            );
            return Ok(());
        }

        app.folder_path = Some(path.clone());
        app.file_paths = files.clone();
        app.current_index = 0;

        // 加载第一张
        if let Some(first) = files.first() {
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
            app.set_status(
                "有未保存的更改，请先保存或重置",
                crate::model::StatusLevel::Warning,
            );
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
            app.set_status(
                "有未保存的更改，请先保存或重置",
                crate::model::StatusLevel::Warning,
            );
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

    /// 触发保存确认（仅设置标志，由 UI 对话框调用 do_save）
    pub fn save_exif(app: &mut AppState) -> Result<()> {
        if !app.has_unsaved_changes() {
            app.set_status("没有需要保存的更改", crate::model::StatusLevel::Info);
            return Ok(());
        }
        app.pending_save = true;
        Ok(())
    }

    /// 执行实际保存操作（由确认对话框调用）
    pub fn do_save(app: &mut AppState) -> Result<()> {
        if let Some(path) = app.current_path() {
            let modified = app.exif_entries.clone();

            // 写入 EXIF
            ExifWriter::write(path, &modified)?;

            app.original_exif = modified;
            app.undo_stack.clear();
            app.set_status("保存成功", crate::model::StatusLevel::Success);
        }

        Ok(())
    }

    /// 重置为原始 EXIF
    pub fn reset_exif(app: &mut AppState) {
        app.exif_entries = app.original_exif.clone();
        app.undo_stack.clear();
        app.selected_tags.clear();
        app.editing_tag = None;
        app.set_status("已重置为原始 EXIF", crate::model::StatusLevel::Info);
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
        let tags: Vec<ExifTag> = app.selected_tags.drain(..).collect();
        for tag in tags {
            if let Some(old_value) = app.exif_entries.remove(&tag) {
                app.push_undo(tag, old_value, ExifValue::Ascii(String::new()));
            }
        }
        app.set_status(
            "已删除选中的字段",
            crate::model::StatusLevel::Success,
        );
    }

    /// 批量清除 EXIF（所有或仅 GPS）
    pub fn clear_exif(app: &mut AppState, mode: ClearMode) {
        let tags_to_remove: Vec<ExifTag> = app
            .exif_entries
            .keys()
            .filter(|tag| match mode {
                ClearMode::All => true,
                ClearMode::GPS => tag.ifd == "GPS",
                ClearMode::Privacy => {
                    tag.ifd == "GPS"
                        || tag.id == 0x8298  // Copyright
                        || tag.id == 0x927C  // MakerNote
                        || tag.id == 0x9286  // UserComment
                        || tag.id == 0xA435  // LensSerialNumber
                }
            })
            .cloned()
            .collect();

        for tag in tags_to_remove {
            if let Some(old_value) = app.exif_entries.remove(&tag) {
                app.push_undo(tag.clone(), old_value, ExifValue::Ascii(String::new()));
            }
        }
        let msg = match mode {
            ClearMode::All => "已清除所有 EXIF 字段",
            ClearMode::GPS => "已清除 GPS 字段",
            ClearMode::Privacy => "已清除隐私字段",
        };
        app.set_status(
            msg,
            crate::model::StatusLevel::Success,
        );
    }

    /// 按 EXIF 重命名当前文件
    pub fn rename_by_exif(app: &mut AppState) -> Result<()> {
        let src = app.current_path().cloned().ok_or_else(|| anyhow::anyhow!("没有打开文件"))?;

        // 尝试从 EXIF 提取日期、ISO、光圈信息
        let mut parts = Vec::new();

        // 拍摄时间
        if let Some(v) = app.exif_entries.values().find(|v| v.to_display_string().contains('-') || v.to_display_string().contains(':')) {
            let s = v.to_display_string().chars().filter(|c| c.is_ascii_digit()).take(14).collect::<String>();
            if s.len() >= 8 {
                parts.push(s.chars().take(8).collect());
            }
        }

        // ISO
        for (tag, _) in &app.exif_entries {
            if tag.id == 0x8827 {
                // ISO
                if let Some(v) = app.exif_entries.get(tag) {
                    let s = v.to_display_string();
                    if !s.is_empty() {
                        parts.push(format!("ISO{}", s));
                    }
                }
            }
        }

        // 光圈
        for (tag, _) in &app.exif_entries {
            if tag.id == 0x829D {
                if let Some(v) = app.exif_entries.get(tag) {
                    let raw = v.to_display_string();
                    if raw.contains('/') {
                        parts.push(format!("f{}", raw));
                    }
                }
            }
        }

        if parts.is_empty() {
            app.set_status("无法从 EXIF 提取信息进行重命名", crate::model::StatusLevel::Warning);
            return Ok(());
        }

        let name = parts.join("_");
        let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
        let parent = src.parent().unwrap_or(std::path::Path::new("."));
        let new_path = parent.join(format!("{}.{}", name, ext));

        if new_path.exists() && new_path != src {
            app.set_status(
                format!("目标文件 {} 已存在", new_path.display()),
                crate::model::StatusLevel::Warning,
            );
            return Ok(());
        }

        std::fs::rename(&src, &new_path)?;

        // 更新文件列表
        if let Some(idx) = app.file_paths.iter().position(|p| p == &src) {
            app.file_paths[idx] = new_path.clone();
        }

        Self::open_file(app, new_path)?;
        app.set_status("文件已重命名", crate::model::StatusLevel::Success);
        Ok(())
    }

    /// 加载文件夹缩略图
    fn load_thumbnails_for_folder(app: &mut AppState) {
        // 生成缩略图（当前图片已加载）
        app.thumbnails.clear();

        for path in &app.file_paths {
            let is_current = app
                .current_path()
                .map(|p| p == path)
                .unwrap_or(false);

            let thumb = ThumbnailInfo {
                path: path.clone(),
                image: if is_current {
                    app.current_image.clone()
                } else {
                    None
                },
                is_loading: !is_current,
            };
            app.thumbnails.push(thumb);
        }
    }

    /// 点击缩略图切换
    pub fn select_thumbnail(app: &mut AppState, index: usize) -> Result<()> {
        if index >= app.file_paths.len() {
            return Ok(());
        }

        if app.has_unsaved_changes() {
            app.set_status(
                "有未保存的更改，请先保存或重置",
                crate::model::StatusLevel::Warning,
            );
            return Ok(());
        }

        app.current_index = index;
        if let Some(path) = app.current_path().cloned() {
            Self::open_file(app, path)?;
        }

        Ok(())
    }
}

pub enum ClearMode {
    All,
    GPS,
    Privacy,
}
