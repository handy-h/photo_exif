use crate::model::AppState;

/// 处理键盘快捷键
pub fn handle_shortcuts(app: &mut AppState, ui: &mut egui::Ui) {
    let input = ui.input(|i| i.clone());

    // ← → 切换照片
    if input.key_pressed(egui::Key::ArrowLeft) {
        let _ = crate::io::FileOps::prev_image(app);
    }
    if input.key_pressed(egui::Key::ArrowRight) {
        let _ = crate::io::FileOps::next_image(app);
    }

    // Ctrl+S 保存（触发确认对话框）
    if input.modifiers.command && input.key_pressed(egui::Key::S) {
        if app.has_unsaved_changes() {
            app.pending_save = true;
        } else {
            app.set_status("没有需要保存的更改", crate::model::StatusLevel::Info);
        }
    }

    // Ctrl+Z 撤销
    if input.modifiers.command && input.key_pressed(egui::Key::Z) {
        if app.undo() {
            app.set_status("已撤销", crate::model::StatusLevel::Info);
        }
    }

    // Ctrl+Shift+C 复制 EXIF
    if input.modifiers.command && input.modifiers.shift && input.key_pressed(egui::Key::C) {
        copy_exif(app);
    }

    // Ctrl+Shift+V 粘贴 EXIF
    if input.modifiers.command && input.modifiers.shift && input.key_pressed(egui::Key::V) {
        paste_exif(app);
    }

    // Ctrl+O 打开文件夹
    if input.modifiers.command && input.key_pressed(egui::Key::O) {
        open_folder_dialog(app);
    }

    // Ctrl+N 打开单个文件
    if input.modifiers.command && input.key_pressed(egui::Key::N) {
        open_file_dialog(app);
    }

    // Del 删除选中字段
    if input.key_pressed(egui::Key::Delete) {
        if !app.selected_tags.is_empty() {
            crate::io::FileOps::delete_selected(app);
        }
    }

    // F 全屏切换
    if input.key_pressed(egui::Key::F) {
        app.is_fullscreen = !app.is_fullscreen;
    }

    // 1 切换 1:1 像素视图
    if input.key_pressed(egui::Key::Num1) {
        app.pixel_perfect = !app.pixel_perfect;
        if app.pixel_perfect {
            app.zoom = 1.0;
        }
    }

    // +/- 缩放
    if input.key_pressed(egui::Key::Plus) || input.key_pressed(egui::Key::Equals) {
        app.zoom = (app.zoom * 1.2).min(10.0);
        app.pixel_perfect = false;
    }
    if input.key_pressed(egui::Key::Minus) {
        app.zoom = (app.zoom / 1.2).max(0.1);
        app.pixel_perfect = false;
    }

    // 滚轮缩放（仅在鼠标位于预览区域时生效）
    let scroll_delta = input.smooth_scroll_delta.y;
    const ZOOM_THRESHOLD: f32 = 10.0;
    if scroll_delta.abs() > ZOOM_THRESHOLD && app.pointer_over_preview {
        let factor = if scroll_delta > 0.0 { 1.1 } else { 1.0 / 1.1 };
        app.zoom = (app.zoom * factor).clamp(0.1, 10.0);
        app.pixel_perfect = false;
    }

    // Ctrl+Shift+R 打开 EXIF 修复
    if input.modifiers.command && input.modifiers.shift && input.key_pressed(egui::Key::R) {
        app.repair_window.active = true;
    }

    // Ctrl+Shift+G 打开 GPX 写入
    if input.modifiers.command && input.modifiers.shift && input.key_pressed(egui::Key::G) {
        app.gpx_window.active = true;
    }

    // Ctrl+Shift+D 打开对比模式
    if input.modifiers.command && input.modifiers.shift && input.key_pressed(egui::Key::D) {
        app.compare_state.active = true;
    }
}

fn copy_exif(app: &mut AppState) {
    if app.exif_entries.is_empty() {
        app.set_status("没有 EXIF 数据可复制", crate::model::StatusLevel::Warning);
        return;
    }
    app.clipboard_exif = Some(app.exif_entries.clone());
    app.set_status(
        format!("已复制 {} 个 EXIF 字段", app.exif_entries.len()),
        crate::model::StatusLevel::Success,
    );
}

fn paste_exif(app: &mut AppState) {
    if let Some(clipboard) = app.clipboard_exif.clone() {
        let count = clipboard.len();
        for (tag, value) in &clipboard {
            if let Some(old) = app.exif_entries.get(tag) {
                app.push_undo(tag.clone(), old.clone(), value.clone());
            } else {
                app.push_undo(tag.clone(), crate::model::ExifValue::Ascii(String::new()), value.clone());
            }
            app.exif_entries.insert(tag.clone(), value.clone());
        }
        app.set_status(
            format!("已粘贴 {} 个 EXIF 字段", count),
            crate::model::StatusLevel::Success,
        );
    } else {
        app.set_status("剪贴板中没有 EXIF 数据", crate::model::StatusLevel::Warning);
    }
}

fn open_folder_dialog(app: &mut AppState) {
    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
        if let Err(e) = crate::io::FileOps::open_folder(app, folder) {
            app.set_status(format!("打开文件夹失败: {}", e), crate::model::StatusLevel::Error);
        }
    }
}

fn open_file_dialog(app: &mut AppState) {
    if let Some(file) = rfd::FileDialog::new()
        .add_filter("图片", &["jpg", "jpeg", "png", "webp", "tiff", "tif", "bmp"])
        .pick_file()
    {
        if let Err(e) = crate::io::FileOps::open_file(app, file) {
            app.set_status(format!("打开文件失败: {}", e), crate::model::StatusLevel::Error);
        }
    }
}
