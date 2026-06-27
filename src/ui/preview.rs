use crate::model::{AppState, ExifGroup, ExifTag, ExifValue};

/// 渲染左侧图片预览面板
pub fn render_preview_panel(app: &mut AppState, ctx: &egui::Context) {
    if app.is_fullscreen {
        render_fullscreen_preview(app, ctx);
        return;
    }

    egui::SidePanel::left("preview_panel")
        .resizable(true)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                render_preview_area(app, ui);
                ui.separator();
                render_navigation_buttons(app, ui);
            });
        });
}

fn render_fullscreen_preview(app: &mut AppState, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(color_img) = &app.current_image {
            let available_size = ui.available_size();
            let img_size = egui::vec2(
                color_img.size[0] as f32 * app.zoom,
                color_img.size[1] as f32 * app.zoom,
            );

            let (rect, _) =
                ui.allocate_exact_size(img_size, egui::Sense::hover());

            let painter = ui.painter();
            painter.image(
                color_img.texture_id(ctx),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            // 全屏时显示信息
            ui.horizontal(|ui| {
                ui.label(format!("{:.0}%", app.zoom * 100.0));
                if ui.button("退出全屏").clicked() {
                    app.is_fullscreen = false;
                }
                if ui.button("上一张").clicked() {
                    let _ = crate::io::FileOps::prev_image(app);
                    ctx.request_repaint();
                }
                if ui.button("下一张").clicked() {
                    let _ = crate::io::FileOps::next_image(app);
                    ctx.request_repaint();
                }
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("没有打开的图片");
            });
        }
    });
}

fn render_preview_area(app: &mut AppState, ui: &mut egui::Ui) {
    if let Some(color_img) = &app.current_image {
        let available_size = ui.available_size();
        let img_w = color_img.size[0] as f32;
        let img_h = color_img.size[1] as f32;

        // 计算适合容器的缩放
        let scale_x = available_size.x / img_w;
        let scale_y = available_size.y / img_h;
        let fit_scale = scale_x.min(scale_y).min(1.0);

        let display_size = egui::vec2(
            img_w * fit_scale * app.zoom,
            img_h * fit_scale * app.zoom,
        );

        let (rect, _) =
            ui.allocate_exact_size(display_size, egui::Sense::hover());

        let painter = ui.painter();
        painter.image(
            color_img.texture_id(ui.ctx()),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );

        // 缩放信息
        ui.horizontal(|ui| {
            ui.label(format!(
                "{} x {} ({:.0}%)",
                img_w as u32,
                img_h as u32,
                fit_scale * app.zoom * 100.0
            ));
            if app.extension_warning.is_some() {
                ui.colored_label(egui::Color32::YELLOW, "⚠ 格式不匹配");
            }
        });
    } else {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.heading("📷");
                ui.label("打开文件夹或图片开始使用");
                ui.label("支持 JPEG、PNG、WebP、TIFF、BMP");
            });
        });
    }
}

fn render_navigation_buttons(app: &mut AppState, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("◀ 上一张").clicked() {
            let _ = crate::io::FileOps::prev_image(app);
        }

        ui.label(format!("{}/{}", app.current_index + 1, app.file_paths.len()));

        if ui.button("下一张 ▶").clicked() {
            let _ = crate::io::FileOps::next_image(app);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("全屏").clicked() {
                app.is_fullscreen = !app.is_fullscreen;
            }
            if ui.button("重置").clicked() {
                crate::io::FileOps::reset_exif(app);
            }
            if ui.button("保存").clicked() {
                let _ = crate::io::FileOps::save_exif(app);
            }
        });
    });
}
