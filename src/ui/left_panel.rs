use crate::model::AppState;

/// 渲染左侧缩略图列表面板
pub fn render_left_panel(app: &mut AppState, ctx: &egui::Context) {
    if app.is_fullscreen {
        return;
    }

    egui::SidePanel::left("left_panel")
        .resizable(true)
        .default_width(120.0)
        .min_width(80.0)
        .max_width(200.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("图片列表");
                ui.separator();

                if app.file_paths.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("(无图片)");
                    });
                    return;
                }

                // 显示当前文件夹信息
                if let Some(folder) = &app.folder_path {
                    ui.label(
                        folder
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    );
                    ui.separator();
                }

                // 缩略图列表
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let paths: Vec<std::path::PathBuf> = app.file_paths.clone();
                    let current_idx = app.current_index;

                    for (i, path) in paths.iter().enumerate() {
                        let is_current = i == current_idx;

                        let frame = if is_current {
                            egui::Frame::group(ui.style())
                                .stroke(egui::Stroke::new(
                                    2.0,
                                    egui::Color32::from_rgb(100, 149, 237),
                                ))
                                .fill(egui::Color32::from_rgb(230, 240, 255))
                        } else {
                            egui::Frame::group(ui.style())
                                .stroke(egui::Stroke::new(1.0, egui::Color32::DARK_GRAY))
                        };

                        let response = frame.show(ui, |ui| {
                            ui.set_min_size(egui::vec2(80.0, 60.0));

                            // 尝试显示缩略图
                            let has_thumb = app
                                .thumbnails
                                .get(i)
                                .map(|t| t.image.is_some())
                                .unwrap_or(false);

                            if has_thumb {
                                if let Some(img) = app.thumbnails[i].image.as_ref() {
                                    let needs_reload =
                                        app.thumbnails[i].texture_id.is_none()
                                            || app.thumbnails[i].image.as_ref() != Some(img);

                                    let texture_id = if needs_reload {
                                        let texture = ctx.load_texture(
                                            format!("thumb_{}", i),
                                            img.clone(),
                                            egui::TextureOptions::LINEAR,
                                        );
                                        app.thumbnails[i].texture_id = Some(texture.id());
                                        texture.id()
                                    } else {
                                        app.thumbnails[i].texture_id.unwrap()
                                    };

                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(80.0, 60.0),
                                        egui::Sense::click(),
                                    );
                                    let painter = ui.painter();
                                    painter.image(
                                        texture_id,
                                        rect,
                                        egui::Rect::from_min_max(
                                            egui::pos2(0.0, 0.0),
                                            egui::pos2(1.0, 1.0),
                                        ),
                                        egui::Color32::WHITE,
                                    );
                                }
                            } else {
                                ui.vertical_centered(|ui| {
                                    ui.label("📷");
                                });
                            }

                            // 文件名
                            if let Some(name) = path.file_name() {
                                let name_str = name.to_string_lossy();
                                let short = if name_str.len() > 12 {
                                    format!("{}...", &name_str[..12])
                                } else {
                                    name_str.to_string()
                                };
                                ui.label(short);
                            }
                        });

                        if response.response.clicked() {
                            let _ = crate::io::FileOps::select_thumbnail(app, i);
                        }
                    }
                });

                ui.separator();

                // 导航按钮
                ui.horizontal(|ui| {
                    if ui.button("◀").clicked() {
                        let _ = crate::io::FileOps::prev_image(app);
                    }
                    ui.label(format!("{}/{}", app.current_index + 1, app.file_paths.len()));
                    if ui.button("▶").clicked() {
                        let _ = crate::io::FileOps::next_image(app);
                    }
                });
            });
        });
}
