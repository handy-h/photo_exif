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
            if ui.button("🕐 最近打开").clicked() {
                // 通过 app 状态传递标志 — 但 AppState 不包含此标志
                // 改为直接在 app.rs 中处理
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

            ui.separator();

            // 批量操作菜单
            ui.menu_button("⚙ 批量操作", |ui| {
                if ui.button("清除所有 EXIF").clicked() {
                    crate::io::FileOps::clear_exif(app, crate::io::ClearMode::All);
                    ui.close_menu();
                }
                if ui.button("清除 GPS 数据").clicked() {
                    crate::io::FileOps::clear_exif(app, crate::io::ClearMode::GPS);
                    ui.close_menu();
                }
                if ui.button("一键清除隐私字段").clicked() {
                    crate::io::FileOps::clear_exif(app, crate::io::ClearMode::Privacy);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("按 EXIF 重命名").clicked() {
                    let _ = crate::io::FileOps::rename_by_exif(app);
                    ui.close_menu();
                }
            });

            ui.separator();

            // 高级功能
            if ui.button("📍 GPX写入").clicked() {
                app.gpx_window.active = true;
            }
            if ui.button("🔀 对比").clicked() {
                app.compare_state.active = true;
            }
            if ui.button("🔧 修复").clicked() {
                app.repair_window.active = true;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 文件名
                if let Some(path) = app.current_path() {
                    ui.label(
                        path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    );
                }
            });
        });
    });
}

fn open_folder_dialog(app: &mut AppState) {
    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
        if let Err(e) = crate::io::FileOps::open_folder(app, folder) {
            app.set_status(
                format!("打开文件夹失败: {}", e),
                crate::model::StatusLevel::Error,
            );
        }
    }
}

fn open_file_dialog(app: &mut AppState) {
    if let Some(file) = rfd::FileDialog::new()
        .add_filter("图片", &["jpg", "jpeg", "png", "webp", "tiff", "tif", "bmp"])
        .pick_file()
    {
        if let Err(e) = crate::io::FileOps::open_file(app, file) {
            app.set_status(
                format!("打开文件失败: {}", e),
                crate::model::StatusLevel::Error,
            );
        }
    }
}
