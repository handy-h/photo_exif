use crate::exif::repair::{ExifRepairer, ExifHealth};
use crate::model::AppState;

/// EXIF 校验和修复窗口状态
#[derive(Debug)]
pub struct RepairWindowState {
    pub health: Option<ExifHealth>,
    pub repair_log: Vec<String>,
    pub active: bool,
}

impl Default for RepairWindowState {
    fn default() -> Self {
        Self {
            health: None,
            repair_log: Vec::new(),
            active: false,
        }
    }
}

/// 渲染 EXIF 校验和修复窗口
pub fn render_repair_window(
    ctx: &egui::Context,
    repair_state: &mut RepairWindowState,
    app: &mut AppState,
) {
    if !repair_state.active {
        return;
    }

    let current_path = app.current_path().cloned();
    let health_clone = repair_state.health.as_ref().map(|h| ExifHealthSnapshot {
        score: h.score,
        format_valid: h.format_valid,
        jfif_valid: h.jfif_valid,
        exif_valid: h.exif_valid,
        marker_valid: h.marker_valid,
    });
    let log_clone = repair_state.repair_log.clone();

    let mut do_validate = false;
    let mut do_repair = false;
    let mut do_clear_log = false;
    let mut should_close = false;

    egui::Window::new("EXIF 校验和修复")
        .open(&mut should_close)
        .default_width(500.0)
        .default_height(400.0)
        .resizable(true)
        .show(ctx, |ui| {
            let _path = match &current_path {
                Some(p) => p,
                None => {
                    ui.centered_and_justified(|ui| {
                        ui.label("请先打开图片");
                    });
                    return;
                }
            };

            ui.horizontal(|ui| {
                if ui.button("🔍 校验当前图片").clicked() {
                    do_validate = true;
                }
                if ui.button("🔧 尝试修复").clicked() {
                    do_repair = true;
                }
            });

            ui.separator();

            if let Some(ref health) = health_clone {
                ui.group(|ui| {
                    ui.heading("健康状态");

                    let score_color = if health.score >= 90.0 {
                        egui::Color32::GREEN
                    } else if health.score >= 70.0 {
                        egui::Color32::YELLOW
                    } else if health.score >= 50.0 {
                        egui::Color32::from_rgb(255, 165, 0)
                    } else {
                        egui::Color32::RED
                    };

                    ui.horizontal(|ui| {
                        ui.label("总体评分:");
                        ui.colored_label(score_color, format!("{:.0}/100", health.score));
                        ui.colored_label(score_color, health.status_label());
                    });

                    ui.add_space(4.0);

                    egui::Grid::new("health_grid")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("文件格式:");
                            status_icon(ui, health.format_valid);
                            ui.end_row();

                            ui.label("JFIF 结构:");
                            status_icon(ui, health.jfif_valid);
                            ui.end_row();

                            ui.label("EXIF 完整性:");
                            status_icon(ui, health.exif_valid);
                            ui.end_row();

                            ui.label("标记结构:");
                            status_icon(ui, health.marker_valid);
                            ui.end_row();
                        });
                });
            }

            ui.add_space(8.0);

            if !log_clone.is_empty() {
                ui.heading("操作日志");
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for (i, log) in log_clone.iter().enumerate() {
                            ui.label(format!("{}. {}", i + 1, log));
                        }
                    });

                if ui.button("清空日志").clicked() {
                    do_clear_log = true;
                }
            }
        });

    // 应用操作
    if should_close {
        repair_state.active = false;
    }
    if do_clear_log {
        repair_state.repair_log.clear();
    }
    if let Some(path) = current_path {
        if do_validate {
            match ExifRepairer::validate(&path) {
                Ok(health) => {
                    repair_state.health = Some(health);
                }
                Err(e) => {
                    repair_state.repair_log.push(format!("校验失败: {}", e));
                }
            }
        }
        if do_repair {
            match ExifRepairer::repair(&path) {
                Ok(result) => {
                    repair_state.repair_log.push(format!(
                        "修复完成: {} 项修复，{} 字节变化",
                        result.repairs.len(),
                        result.bytes_saved
                    ));
                    for r in &result.repairs {
                        repair_state.repair_log.push(format!("  • {}", r));
                    }
                    if let Ok(health) = ExifRepairer::validate(&path) {
                        repair_state.health = Some(health);
                    }
                    let _ = crate::io::FileOps::open_file(app, path.clone());
                }
                Err(e) => {
                    repair_state.repair_log.push(format!("修复失败: {}", e));
                }
            }
        }
    }
}

#[derive(Clone)]
struct ExifHealthSnapshot {
    score: f32,
    format_valid: bool,
    jfif_valid: bool,
    exif_valid: bool,
    marker_valid: bool,
}

impl ExifHealthSnapshot {
    fn status_label(&self) -> &'static str {
        if self.score >= 90.0 {
            "✅ 良好"
        } else if self.score >= 70.0 {
            "⚠️ 轻微损坏"
        } else if self.score >= 50.0 {
            "⚠️ 损坏"
        } else {
            "❌ 严重损坏"
        }
    }
}

fn status_icon(ui: &mut egui::Ui, ok: bool) {
    if ok {
        ui.colored_label(egui::Color32::GREEN, "✅ 正常");
    } else {
        ui.colored_label(egui::Color32::RED, "❌ 异常");
    }
}
