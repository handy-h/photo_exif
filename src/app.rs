use crate::model::AppState;
use crate::ui::{render_exif_panel, render_preview_panel, render_toolbar, handle_shortcuts};
use eframe::egui;

/// 主应用结构
pub struct PhotoExitApp {
    state: AppState,
}

impl PhotoExitApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // 尝试加载上次的文件夹
        let settings = crate::config::Settings::load();
        let mut state = AppState::new();

        if let Some(folder) = settings.last_folder {
            if folder.exists() {
                let _ = crate::io::FileOps::open_folder(&mut state, folder);
            }
        }

        Self { state }
    }
}

impl eframe::App for PhotoExitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        handle_shortcuts(&mut self.state, ctx);

        render_toolbar(&mut self.state, ctx);
        render_preview_panel(&mut self.state, ctx);
        render_exif_panel(&mut self.state, ctx);

        // 底部状态栏
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("photo_exit v0.1.0"));
                ui.separator();
                ui.label("← → 切换 | Ctrl+S 保存 | F 全屏 | +/- 缩放 | 1 实际像素");
            });
        });

        // 处理拖放
        handle_drop(&mut self.state, ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let mut settings = crate::config::Settings::load();
        if let Some(folder) = &self.state.folder_path {
            settings.last_folder = Some(folder.clone());
        }
        let _ = settings.save();
    }
}

fn handle_drop(app: &mut AppState, ctx: &egui::Context) {
    if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
        for file in ctx.input(|i| i.raw.dropped_files.clone()) {
            if let Some(path) = file.path {
                if path.is_dir() {
                    let _ = crate::io::FileOps::open_folder(app, path);
                } else {
                    let _ = crate::io::FileOps::open_file(app, path);
                }
            }
        }
    }
}
