use crate::model::AppState;

/// 渲染中央图片预览区（主预览区域）
pub fn render_preview_panel(app: &mut AppState, ctx: &egui::Context) {
    if app.is_fullscreen {
        render_fullscreen_preview(app, ctx);
        return;
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(color_img) = &app.current_image {
            let available_size = ui.available_size();
            let img_w = color_img.size[0] as f32;
            let img_h = color_img.size[1] as f32;

            // 获取或创建纹理
            let texture = ui.ctx().load_texture(
                "current_image",
                color_img.clone(),
                egui::TextureOptions::LINEAR,
            );

            // 计算显示尺寸：适应容器或 1:1 像素
            let (display_size, scale_label) = if app.pixel_perfect {
                (
                    egui::vec2(img_w, img_h),
                    format!("1:1 ({} x {})", img_w as u32, img_h as u32),
                )
            } else {
                let scale_x = available_size.x / img_w;
                let scale_y = available_size.y / img_h;
                let fit_scale = scale_x.min(scale_y).min(1.0);
                (
                    egui::vec2(
                        img_w * fit_scale * app.zoom,
                        img_h * fit_scale * app.zoom,
                    ),
                    format!(
                        "{} x {} ({:.0}%)",
                        img_w as u32,
                        img_h as u32,
                        fit_scale * app.zoom * 100.0
                    ),
                )
            };

            // 居中显示图片
            let (rect, _) = ui.allocate_exact_size(display_size, egui::Sense::hover());

            // 绘制图片
            let painter = ui.painter();
            painter.image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            // 在左下角显示缩放信息
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.label(scale_label);
                    if app.pixel_perfect {
                        ui.colored_label(egui::Color32::LIGHT_BLUE, "1:1");
                    }
                    if app.extension_warning.is_some() {
                        ui.colored_label(egui::Color32::YELLOW, "⚠ 格式不匹配");
                    }
                });
            });
        } else {
            // 空状态
            let available = ui.available_size();
            let (rect, _) = ui.allocate_exact_size(available, egui::Sense::hover());

            // 绘制浅灰色背景
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_gray(240));

            // 绘制边框
            ui.painter().rect_stroke(
                rect.shrink(1.0),
                0.0,
                egui::Stroke::new(1.0, egui::Color32::from_gray(200)),
            );

            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("📷");
                    ui.add_space(8.0);
                    ui.label("打开文件夹或图片开始使用");
                    ui.add_space(4.0);
                    ui.label("支持 JPEG、PNG、WebP、TIFF、BMP");
                    ui.add_space(16.0);
                    ui.label("💡 提示：拖拽文件到此处也可打开");
                });
            });
        }
    });
}

fn render_fullscreen_preview(app: &mut AppState, ctx: &egui::Context) {
    // 提取图片信息避免借用冲突
    let img_info = app
        .current_image
        .as_ref()
        .map(|img| (img.size[0], img.size[1], img.clone()));

    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some((w, h, color_img)) = img_info {
            // 获取或创建纹理
            let texture = ctx.load_texture(
                "current_image",
                color_img,
                egui::TextureOptions::LINEAR,
            );
            let img_size = if app.pixel_perfect {
                egui::vec2(w as f32, h as f32)
            } else {
                let available_size = ui.available_size();
                let scale_x = available_size.x / w as f32;
                let scale_y = available_size.y / h as f32;
                let fit_scale = scale_x.min(scale_y).min(1.0);
                egui::vec2(
                    w as f32 * fit_scale * app.zoom,
                    h as f32 * fit_scale * app.zoom,
                )
            };

            let (rect, _) = ui.allocate_exact_size(img_size, egui::Sense::hover());

            let painter = ui.painter();
            painter.image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            // 全屏时显示信息
            ui.horizontal(|ui| {
                let label = if app.pixel_perfect {
                    format!("1:1 ({} x {})", w, h)
                } else {
                    format!("{:.0}%", app.zoom * 100.0)
                };
                ui.label(label);
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
            // 全屏模式空状态
            let available = ui.available_size();
            let (rect, _) = ui.allocate_exact_size(available, egui::Sense::hover());

            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_gray(240));

            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("📷 Photo EXIF Tool");
                    ui.add_space(16.0);
                    ui.label("按 Ctrl+O 打开文件夹 或 Ctrl+N 打开文件");
                    ui.add_space(8.0);
                    ui.label("拖拽图片到窗口也可打开");
                });
            });
        }
    });
}
