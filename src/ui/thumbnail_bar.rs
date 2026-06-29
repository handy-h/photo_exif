use crate::model::AppState;

/// 渲染底部缩略图画廊
pub fn render_thumbnail_bar(app: &mut AppState, ctx: &egui::Context) {
    if app.is_fullscreen || app.thumbnails.is_empty() {
        return;
    }

    egui::TopBottomPanel::bottom("thumbnail_bar")
        .resizable(true)
        .default_height(90.0)
        .min_height(60.0)
        .max_height(150.0)
        .show(ctx, |ui| {
            egui::ScrollArea::horizontal()
                .id_source("thumb_scroll")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let thumb_size = 70.0_f32;
                        let current_idx = app.current_index;

                        let paths: Vec<std::path::PathBuf> = app
                            .thumbnails
                            .iter()
                            .map(|t| t.path.clone())
                            .collect();

                        for (i, path) in paths.iter().enumerate() {
                            let is_current = i == current_idx;
                            let has_image = app.thumbnails.get(i).map(|t| t.image.is_some()).unwrap_or(false);

                            let frame = if is_current {
                                egui::Frame::group(ui.style())
                                    .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 149, 237)))
                                    .fill(egui::Color32::from_rgb(30, 30, 40))
                            } else {
                                egui::Frame::group(ui.style())
                                    .stroke(egui::Stroke::new(1.0, egui::Color32::DARK_GRAY))
                            };

                            let response = frame.show(ui, |ui| {
                                ui.set_min_size(egui::vec2(thumb_size, thumb_size));
                                ui.set_max_size(egui::vec2(thumb_size, thumb_size));

                                if has_image {
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
                                            egui::vec2(thumb_size, thumb_size),
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
                                        if let Some(name) = path.file_name() {
                                            let name_str = name.to_string_lossy();
                                            let short = if name_str.len() > 10 {
                                                format!("{}...", &name_str[..10])
                                            } else {
                                                name_str.to_string()
                                            };
                                            ui.label(short);
                                        }
                                    });
                                }
                            });

                            if response.response.clicked() {
                                let _ = crate::io::FileOps::select_thumbnail(app, i);
                            }

                            ui.separator();
                        }
                    });
                });
        });
}
