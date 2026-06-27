use crate::model::AppState;

/// 顶部工具栏
pub fn render_toolbar(app: &mut AppState, ctx: &egui::Context) {
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("📁 打开文件夹").clicked() {
                open_folder_dialog(app);
            }
            if ui.button("📄 打开文件").clicked() {
                open_file_dialog(app);
            }
            ui.separator();

            if ui.button("◀").clicked() {
                let _ = crate::io::FileOps::prev_image(app);
            }
            if ui.button("▶").clicked() {
                let _ = crate::io::FileOps::next_image(app);
            }
            ui.separator();

            if ui.button("💾 保存").clicked() {
                let _ = crate::io::FileOps::save_exif(app);
            }
            if ui.button("🔄 重置").clicked() {
                crate::io::FileOps::reset_exif(app);
            }
            ui.separator();

            if ui.button("🗑 删除选中").clicked() {
                crate::io::FileOps::delete_selected(app);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 状态消息
                if let Some((msg, level)) = &app.status_message {
                    let color = match level {
                        crate::model::StatusLevel::Info => egui::Color32::LIGHT_BLUE,
                        crate::model::StatusLevel::Success => egui::Color32::GREEN,
                        crate::model::StatusLevel::Warning => egui::Color32::YELLOW,
                        crate::model::StatusLevel::Error => egui::Color32::RED,
                    };
                    ui.colored_label(color, msg);
                }

                // 文件名
                if let Some(path) = app.current_path() {
                    ui.label(path.file_name().unwrap_or_default().to_string_lossy().to_string());
                }
            });
        });
    });
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
