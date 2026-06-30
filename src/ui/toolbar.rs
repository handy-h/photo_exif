use crate::model::AppState;
use egui::containers::panel::Panel;

/// 顶部工具栏
pub fn render_toolbar(app: &mut AppState, ui: &mut egui::Ui) {
    Panel::top("toolbar").show(ui, |ui| {
        ui.horizontal(|ui| {
            if ui.button("📁 打开文件夹").clicked() {
                open_folder_dialog(app);
            }
            if ui.button("📄 打开文件").clicked() {
                open_file_dialog(app);
            }
            ui.separator();
            if ui.button("最近").clicked() {
                app.show_recent_menu = true;
            }
        });
    });
}

fn open_folder_dialog(app: &mut AppState) {
    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        match crate::io::FileOps::open_folder(app, path) {
            Ok(_) => app.set_status("已打开文件夹", crate::model::StatusLevel::Success),
            Err(e) => app.set_status(
                &format!("打开文件夹失败: {}", e),
                crate::model::StatusLevel::Error,
            ),
        }
    }
}

fn open_file_dialog(app: &mut AppState) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("图片", &["jpg", "jpeg", "png", "webp", "tiff", "tif", "bmp"])
        .pick_file()
    {
        match crate::io::FileOps::open_file(app, path) {
            Ok(_) => app.set_status("已打开文件", crate::model::StatusLevel::Success),
            Err(e) => app.set_status(
                &format!("打开文件失败: {}", e),
                crate::model::StatusLevel::Error,
            ),
        }
    }
}
