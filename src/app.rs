use crate::model::AppState;
use crate::ui::{
    render_compare_window, render_exif_panel, render_gpx_window, render_preview_panel,
    render_repair_window, render_thumbnail_bar, render_toolbar, handle_shortcuts,
    render_left_panel,
};
use eframe::egui;

/// ��Ӧ�ýṹ
pub struct PhotoExitApp {
    state: AppState,
    settings: crate::config::Settings,
    show_recent_menu: bool,
}

impl PhotoExitApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let settings = crate::config::Settings::load();
        let mut state = AppState::new();

        // �Զ��ָ��ϴδ򿪵��ļ��к�λ��
        if settings.auto_restore {
            if let Some(ref folder) = settings.last_folder {
                if folder.exists() {
                    let _ = crate::io::FileOps::open_folder(&mut state, folder.clone());
                    // �ָ�λ��
                    if settings.last_position > 0
                        && settings.last_position < state.file_paths.len()
                    {
                        state.current_index = settings.last_position;
                        if let Some(path) = state.current_path().cloned() {
                            let _ = crate::io::FileOps::open_file(&mut state, path);
                        }
                    }
                }
            }
        }

        Self {
            state,
            settings,
            show_recent_menu: false,
        }
    }
}

impl eframe::App for PhotoExitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        handle_shortcuts(&mut self.state, ctx);

        // �ر� egui ����ģʽ��������ֺ��͵�������
        let mut style = (*ctx.style()).clone();
        style.debug.debug_on_hover = false;
        style.debug.debug_on_hover_with_all_modifiers = false;
        style.debug.show_widget_hits = false;
        ctx.set_style(style);

        // 1. ����������
        render_toolbar(&mut self.state, ctx);
        
        // 2. �ײ�״̬���������� CentralPanel ֮ǰ��
        self.render_status_bar(ctx);
        
        // 3. �ײ�����ͼ���ȣ������� CentralPanel ֮ǰ��
        render_thumbnail_bar(&mut self.state, ctx);
        
        // 4. ������
        render_left_panel(&mut self.state, ctx);
        
        // 5. �Ҳ����
        render_exif_panel(&mut self.state, ctx);
        
        // 6. ������壨���������Ⱦ��
        render_preview_panel(&mut self.state, ctx);

        // ����򿪲˵�
        self.render_recent_menu(ctx);

        // ��������
        let (compare, gpx, repair) = (
            std::mem::take(&mut self.state.compare_state),
            std::mem::take(&mut self.state.gpx_window),
            std::mem::take(&mut self.state.repair_window),
        );
        let mut compare = compare;
        let mut gpx = gpx;
        let mut repair = repair;

        render_compare_window(ctx, &mut compare, &mut self.state);
        render_gpx_window(ctx, &mut gpx, &mut self.state);
        render_repair_window(ctx, &mut repair, &mut self.state);

        self.state.compare_state = compare;
        self.state.gpx_window = gpx;
        self.state.repair_window = repair;

        // �����Ϸ�
        self.handle_drop(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // ���浱ǰ״̬
        if let Some(folder) = &self.state.folder_path {
            self.settings.last_folder = Some(folder.clone());
            self.settings.last_position = self.state.current_index;
            self.settings.add_recent_folder(folder.clone());
        }
        // ���浱ǰ�ļ�������ļ�
        if let Some(path) = self.state.current_path() {
            self.settings.add_recent_file(path.clone());
        }
        if let Err(e) = self.settings.save() {
            eprintln!("[photo_exif] Failed to save settings on exit: {}", e);
        }
    }
}

impl PhotoExitApp {
    /// ��Ⱦ�ײ�״̬��
    fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("photo_exit v0.1.1");
                ui.separator();
                if let Some(path) = self.state.current_path() {
                    ui.label(
                        path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    );
                    ui.separator();
                }
                ui.label(format!(
                    "{}/{}",
                    self.state.current_index + 1,
                    self.state.file_paths.len()
                ));
                ui.separator();

                // ������ڰ�ť
                if ui.button("?? �Ա�").clicked() {
                    self.state.compare_state.active = true;
                }
                if ui.button("?? GPXд��").clicked() {
                    self.state.gpx_window.active = true;
                }
                if ui.button("?? EXIF�޸�").clicked() {
                    self.state.repair_window.active = true;
                }

                ui.separator();
                ui.label("�� �� �л� | Ctrl+S ���� | F ȫ�� | +/- ���� | 1 ʵ������ | Ctrl+Z ���� | Del ɾ��");

                // ״̬��Ϣ
                if let Some((msg, level)) = &self.state.status_message {
                    let color = match level {
                        crate::model::StatusLevel::Info => egui::Color32::LIGHT_BLUE,
                        crate::model::StatusLevel::Success => egui::Color32::GREEN,
                        crate::model::StatusLevel::Warning => egui::Color32::YELLOW,
                        crate::model::StatusLevel::Error => egui::Color32::RED,
                    };
                    ui.separator();
                    ui.colored_label(color, msg);
                }
            });
        });
    }

    /// �����ļ��Ϸ�
    fn handle_drop(&mut self, ctx: &egui::Context) {
        let dropped: Vec<egui::DroppedFile> = ctx.input(|i| i.raw.dropped_files.clone());
        if dropped.is_empty() {
            return;
        }

        for file in &dropped {
            if let Some(ref path) = file.path {
                if path.is_dir() {
                    // �Ϸ��ļ���
                    match crate::io::FileOps::open_folder(&mut self.state, path.clone()) {
                        Ok(_) => {
                            self.settings.add_recent_folder(path.clone());
                            self.state.set_status(
                                format!("�Ѵ��ļ���: {}", path.display()),
                                crate::model::StatusLevel::Success,
                            );
                        }
                        Err(e) => {
                            self.state.set_status(
                                format!("���ļ���ʧ��: {}", e),
                                crate::model::StatusLevel::Error,
                            );
                        }
                    }
                } else {
                    // �Ϸ��ļ�
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.to_lowercase())
                        .unwrap_or_default();

                    let supported = ["jpg", "jpeg", "png", "webp", "tiff", "tif", "bmp"];
                    if supported.contains(&ext.as_str()) {
                        // �����ǰû�д��ļ��У����Դ������ļ���
                        if self.state.folder_path.is_none() {
                            if let Some(parent) = path.parent() {
                                let _ = crate::io::FileOps::open_folder(
                                    &mut self.state,
                                    parent.to_path_buf(),
                                );
                            }
                        }

                        match crate::io::FileOps::open_file(&mut self.state, path.clone()) {
                            Ok(_) => {
                                self.settings.add_recent_file(path.clone());
                                self.state.set_status(
                                    format!("�Ѵ�: {}", path.display()),
                                    crate::model::StatusLevel::Success,
                                );
                            }
                            Err(e) => {
                                self.state.set_status(
                                    format!("���ļ�ʧ��: {}", e),
                                    crate::model::StatusLevel::Error,
                                );
                            }
                        }
                    } else if ext == "gpx" {
                        // �Ϸ� GPX �ļ�
                        match self.state.gpx_window.load_gpx(path.clone()) {
                            Ok(_) => {
                                self.state.gpx_window.active = true;
                                self.state.set_status(
                                    "GPX �ļ��Ѽ���",
                                    crate::model::StatusLevel::Success,
                                );
                            }
                            Err(e) => {
                                self.state.set_status(
                                    format!("���� GPX ʧ��: {}", e),
                                    crate::model::StatusLevel::Error,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// ��Ⱦ����򿪲˵�
    fn render_recent_menu(&mut self, ctx: &egui::Context) {
        if !self.show_recent_menu {
            return;
        }

        // ��ȡ���ݱ�����ó�ͻ
        let recent_folders: Vec<std::path::PathBuf> = self.settings.valid_recent_folders().into_iter().cloned().collect();
        let recent_files: Vec<std::path::PathBuf> = self.settings.valid_recent_files().into_iter().cloned().collect();

        let mut action: Option<RecentAction> = None;

        egui::Window::new("�����")
            .open(&mut self.show_recent_menu)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                // ����ļ���
                ui.heading("����ļ���");
                if recent_folders.is_empty() {
                    ui.colored_label(egui::Color32::DARK_GRAY, "(��)");
                } else {
                    for folder in &recent_folders {
                        let name = folder
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let path_str = folder.display().to_string();
                        if ui.button(&name).on_hover_text(&path_str).clicked() {
                            action = Some(RecentAction::OpenFolder(folder.clone()));
                        }
                    }
                }

                ui.add_space(8.0);

                // ����ļ�
                ui.heading("����ļ�");
                if recent_files.is_empty() {
                    ui.colored_label(egui::Color32::DARK_GRAY, "(��)");
                } else {
                    for file in &recent_files {
                        let name = file
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let path_str = file.display().to_string();
                        if ui.button(&name).on_hover_text(&path_str).clicked() {
                            action = Some(RecentAction::OpenFile(file.clone()));
                        }
                    }
                }

                ui.add_space(8.0);
                ui.separator();
                ui.checkbox(&mut self.settings.auto_restore, "���ʱ�Զ��ָ��ϴλỰ");
            });

        if let Some(act) = action {
            match act {
                RecentAction::OpenFolder(path) => {
                    let _ = crate::io::FileOps::open_folder(&mut self.state, path);
                    self.show_recent_menu = false;
                }
                RecentAction::OpenFile(path) => {
                    let _ = crate::io::FileOps::open_file(&mut self.state, path);
                    self.show_recent_menu = false;
                }
            }
        }
    }
}

enum RecentAction {
    OpenFolder(std::path::PathBuf),
    OpenFile(std::path::PathBuf),
}
