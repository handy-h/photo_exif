use crate::model::AppState;

/// 处理键盘快捷键
pub fn handle_shortcuts(app: &mut AppState, ctx: &egui::Context) {
    let input = ctx.input(|i| i.clone());

    if input.key_pressed(egui::Key::ArrowLeft) {
        let _ = crate::io::FileOps::prev_image(app);
    }
    if input.key_pressed(egui::Key::ArrowRight) {
        let _ = crate::io::FileOps::next_image(app);
    }
    if input.modifiers.command && input.key_pressed(egui::Key::S) {
        let _ = crate::io::FileOps::save_exif(app);
    }
    if input.modifiers.command && input.key_pressed(egui::Key::Z) {
        app.undo();
    }
    if input.key_pressed(egui::Key::Delete) {
        if !app.selected_tags.is_empty() {
            crate::io::FileOps::delete_selected(app);
        }
    }
    if input.key_pressed(egui::Key::F) {
        app.is_fullscreen = !app.is_fullscreen;
    }
    if input.key_pressed(egui::Key::Num1) {
        app.zoom = 1.0;
    }
    if input.key_pressed(egui::Key::Plus) || input.key_pressed(egui::Key::Equals) {
        app.zoom = (app.zoom * 1.2).min(10.0);
    }
    if input.key_pressed(egui::Key::Minus) {
        app.zoom = (app.zoom / 1.2).max(0.1);
    }
    if input.modifiers.command && input.key_pressed(egui::Key::O) {
        open_folder_dialog(app);
    }
}

fn open_folder_dialog(app: &mut AppState) {
    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
        if let Err(e) = crate::io::FileOps::open_folder(app, folder) {
            app.set_status(format!("打开文件夹失败: {}", e), crate::model::StatusLevel::Error);
        }
    }
}
