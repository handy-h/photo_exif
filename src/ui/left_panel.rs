use crate::model::AppState;
use egui::containers::panel::Panel;

/// 渲染左侧面板
pub fn render_left_panel(app: &mut AppState, ui: &mut egui::Ui) {
    if app.is_fullscreen {
        return;
    }

    Panel::left("left_panel")
        .resizable(true)
        .default_size(200.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading("浏览");

                if let Some(ref folder) = app.folder_path {
                    ui.label(
                        folder
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    );

                    ui.add_space(4.0);

                    // 文件列表
                    egui::ScrollArea::vertical()
                        .show(ui, |ui| {
                            let mut to_open: Option<usize> = None;

                            for (i, path) in app.file_paths.iter().enumerate() {
                                let name = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();

                                let is_current = i == app.current_index;
                                let label = if is_current {
                                    egui::RichText::new(&name).strong()
                                } else {
                                    egui::RichText::new(&name)
                                };

                                if ui.selectable_label(is_current, label).clicked() {
                                    to_open = Some(i);
                                }
                            }

                            if let Some(i) = to_open {
                                app.current_index = i;
                                if let Some(path) = app.file_paths.get(i).cloned() {
                                    let _ = crate::io::FileOps::open_file(app, path);
                                }
                            }
                        });
                } else {
                    ui.colored_label(egui::Color32::DARK_GRAY, "未打开文件夹");
                    ui.add_space(8.0);
                    ui.label("拖放图片或文件夹到这里");
                    ui.label("或使用工具栏按钮打开");
                }
            });
        });
}
