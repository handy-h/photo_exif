use crate::model::AppState;
use egui::containers::panel::Panel;

/// 渲染底部缩略图画廊
pub fn render_thumbnail_bar(app: &mut AppState, ui: &mut egui::Ui) {
    if app.is_fullscreen || app.thumbnails.is_empty() {
        return;
    }

    Panel::bottom("thumbnail_bar")
        .resizable(true)
        .default_size(90.0)
        .show(ui, |ui| {
            egui::ScrollArea::horizontal()
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let thumb_size = 70.0_f32;
                        let current_idx = app.current_index;

                        let offset = (app.thumbnail_scroll as usize).max(0);
                        // Collect indices and paths upfront to avoid borrow conflicts
                        let clicks: Vec<(usize, bool)> = app
                            .thumbnails
                            .iter()
                            .enumerate()
                            .skip(offset)
                            .take(20)
                            .map(|(i_offset, t)| {
                                let is_current = i_offset == current_idx;
                                let tint = if is_current {
                                    egui::Color32::from_rgb(255, 200, 50)
                                } else {
                                    egui::Color32::WHITE
                                };

                                let response = egui::Frame::NONE
                                    .fill(if is_current {
                                        egui::Color32::from_rgb(60, 100, 200)
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    })
                                    .show(ui, |ui| {
                                        if let Some(texture_id) = t.texture_id {
                                            ui.add(
                                                egui::Image::new(egui::ImageSource::Texture(
                                                    egui::load::SizedTexture::new(
                                                        texture_id,
                                                        egui::vec2(thumb_size, thumb_size),
                                                    ),
                                                ))
                                                .tint(tint),
                                            );
                                        } else {
                                            let _ = ui.allocate_exact_size(
                                                egui::vec2(thumb_size, thumb_size),
                                                egui::Sense::click(),
                                            );
                                        }
                                    });

                                (i_offset, response.response.clicked())
                            })
                            .filter(|(_, clicked)| *clicked)
                            .collect();

                        for (i_offset, _) in clicks {
                            app.current_index = i_offset;
                            if let Some(path) = app.file_paths.get(i_offset).cloned() {
                                let _ = crate::io::FileOps::open_file(app, path);
                            }
                        }
                    });
                });
        });
}
