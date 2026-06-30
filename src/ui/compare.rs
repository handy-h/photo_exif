use crate::model::{AppState, ExifTag, ExifValue};
use crate::exif::ExifReader;
use crate::exif::ExifFormatter;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// 对比模式状态
#[derive(Debug)]
pub struct CompareState {
    pub left_path: Option<PathBuf>,
    pub right_path: Option<PathBuf>,
    pub left_entries: std::collections::HashMap<ExifTag, ExifValue>,
    pub right_entries: std::collections::HashMap<ExifTag, ExifValue>,
    pub show_only_diff: bool,
    pub active: bool,
}

impl Default for CompareState {
    fn default() -> Self {
        Self {
            left_path: None,
            right_path: None,
            left_entries: Default::default(),
            right_entries: Default::default(),
            show_only_diff: false,
            active: false,
        }
    }
}

impl CompareState {
    /// 加载左侧图片
    pub fn load_left(&mut self, path: PathBuf) -> Result<()> {
        let (entries, _) = ExifReader::read(&path)?;
        self.left_entries = entries;
        self.left_path = Some(path);
        Ok(())
    }

    /// 加载右侧图片
    pub fn load_right(&mut self, path: PathBuf) -> Result<()> {
        let (entries, _) = ExifReader::read(&path)?;
        self.right_entries = entries;
        self.right_path = Some(path);
        Ok(())
    }

    /// 获取所有 tag 的并集（排序）
    pub fn all_tags(&self) -> Vec<ExifTag> {
        let mut tags: BTreeSet<(u16, String, String)> = BTreeSet::new();

        for tag in self.left_entries.keys() {
            tags.insert((tag.id, tag.ifd.clone(), tag.name.clone()));
        }
        for tag in self.right_entries.keys() {
            tags.insert((tag.id, tag.ifd.clone(), tag.name.clone()));
        }

        tags.into_iter()
            .map(|(id, ifd, name)| ExifTag::new(id, &ifd, &name))
            .collect()
    }

    /// 获取差异列表
    pub fn differences(&self) -> Vec<DiffEntry> {
        let mut diffs = Vec::new();
        let all_tags = self.all_tags();

        for tag in &all_tags {
            let left = self.left_entries.get(tag);
            let right = self.right_entries.get(tag);

            match (left, right) {
                (Some(l), Some(r)) => {
                    if l != r {
                        diffs.push(DiffEntry {
                            tag: tag.clone(),
                            left: Some(l.clone()),
                            right: Some(r.clone()),
                            diff_type: DiffType::Modified,
                        });
                    }
                }
                (Some(l), None) => {
                    diffs.push(DiffEntry {
                        tag: tag.clone(),
                        left: Some(l.clone()),
                        right: None,
                        diff_type: DiffType::OnlyLeft,
                    });
                }
                (None, Some(r)) => {
                    diffs.push(DiffEntry {
                        tag: tag.clone(),
                        left: None,
                        right: Some(r.clone()),
                        diff_type: DiffType::OnlyRight,
                    });
                }
                (None, None) => {}
            }
        }

        diffs
    }
}

/// 差异类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffType {
    Modified,
    OnlyLeft,
    OnlyRight,
}

impl DiffType {
    pub fn label(&self) -> &'static str {
        match self {
            DiffType::Modified => "修改",
            DiffType::OnlyLeft => "仅左侧",
            DiffType::OnlyRight => "仅右侧",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            DiffType::Modified => egui::Color32::from_rgb(255, 165, 0),
            DiffType::OnlyLeft => egui::Color32::from_rgb(100, 149, 237),
            DiffType::OnlyRight => egui::Color32::from_rgb(100, 200, 100),
        }
    }
}

/// 单条差异
#[derive(Debug)]
pub struct DiffEntry {
    pub tag: ExifTag,
    pub left: Option<ExifValue>,
    pub right: Option<ExifValue>,
    pub diff_type: DiffType,
}

/// 渲染对比模式窗口
pub fn render_compare_window(
    ui: &mut egui::Ui,
    compare: &mut CompareState,
    app: &mut AppState,
) {
    if !compare.active {
        return;
    }

    // 预计算差异和标签
    let diffs = compare.differences();
    let all_tags = compare.all_tags();
    let show_only_diff = compare.show_only_diff;
    let left_path_name = compare
        .left_path
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string());
    let right_path_name = compare
        .right_path
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string());
    let has_both = compare.left_path.is_some() || compare.right_path.is_some();

    let modified = diffs
        .iter()
        .filter(|d| d.diff_type == DiffType::Modified)
        .count();
    let only_left = diffs
        .iter()
        .filter(|d| d.diff_type == DiffType::OnlyLeft)
        .count();
    let only_right = diffs
        .iter()
        .filter(|d| d.diff_type == DiffType::OnlyRight)
        .count();

    // 预构建显示行
    let tags_to_show: Vec<ExifTag> = if show_only_diff {
        diffs.iter().map(|d| d.tag.clone()).collect()
    } else {
        all_tags.clone()
    };

    let rows: Vec<(String, Option<String>, Option<String>, Option<DiffType>)> = tags_to_show
        .iter()
        .map(|tag| {
            let left_val = compare
                .left_entries
                .get(tag)
                .map(|v| ExifFormatter::format(tag.id, v));
            let right_val = compare
                .right_entries
                .get(tag)
                .map(|v| ExifFormatter::format(tag.id, v));
            let diff_type = diffs.iter().find(|d| &d.tag == tag).map(|d| d.diff_type);
            (tag.name.clone(), left_val, right_val, diff_type)
        })
        .collect();

    // 收集操作
    let mut load_left_file: Option<PathBuf> = None;
    let mut load_right_file: Option<PathBuf> = None;
    let mut load_left_current = false;
    let mut load_right_current = false;
    let mut toggle_show_only_diff = false;
    let mut should_close = false;

    egui::Window::new("EXIF 对比模式")
        .open(&mut should_close)
        .default_width(800.0)
        .default_height(600.0)
        .resizable(true)
        .show(ui.ctx(), |ui| {
            // 左侧选择
            ui.horizontal(|ui| {
                ui.label("左侧:");
                if let Some(ref name) = left_path_name {
                    ui.label(name);
                } else {
                    ui.label("(未选择)");
                }
                if ui.button("选择...").clicked() {
                    load_left_file = rfd::FileDialog::new()
                        .add_filter("图片", &["jpg", "jpeg", "png", "tiff", "tif", "bmp"])
                        .pick_file();
                }
                if ui.button("使用当前图片").clicked() {
                    load_left_current = true;
                }
            });

            // 右侧选择
            ui.horizontal(|ui| {
                ui.label("右侧:");
                if let Some(ref name) = right_path_name {
                    ui.label(name);
                } else {
                    ui.label("(未选择)");
                }
                if ui.button("选择...").clicked() {
                    load_right_file = rfd::FileDialog::new()
                        .add_filter("图片", &["jpg", "jpeg", "png", "tiff", "tif", "bmp"])
                        .pick_file();
                }
                if ui.button("使用当前图片").clicked() {
                    load_right_current = true;
                }
            });

            ui.separator();

            // 统计
            ui.horizontal(|ui| {
                ui.label(format!("共 {} 个字段", all_tags.len()));
                ui.separator();
                ui.colored_label(DiffType::Modified.color(), format!("修改: {}", modified));
                ui.colored_label(DiffType::OnlyLeft.color(), format!("仅左: {}", only_left));
                ui.colored_label(DiffType::OnlyRight.color(), format!("仅右: {}", only_right));
                ui.separator();
                if ui.checkbox(&mut { show_only_diff }, "仅显示差异").changed() {
                    toggle_show_only_diff = true;
                }
            });

            ui.separator();

            if !has_both {
                ui.centered_and_justified(|ui| {
                    ui.label("请选择两张图片进行对比");
                });
                return;
            }

            // 表格
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("compare_grid")
                    .num_columns(4)
                    .spacing([10.0, 4.0])
                    .striped(true)
                    .min_col_width(80.0)
                    .show(ui, |ui| {
                        ui.heading("字段");
                        ui.heading("左侧");
                        ui.heading("右侧");
                        ui.heading("差异");
                        ui.end_row();

                        for (name, left_val, right_val, diff_type) in &rows {
                            ui.label(name);

                            if let Some(val) = left_val {
                                if let Some(dt) = diff_type {
                                    ui.colored_label(dt.color(), val);
                                } else {
                                    ui.label(val);
                                }
                            } else {
                                ui.colored_label(egui::Color32::DARK_GRAY, "—");
                            }

                            if let Some(val) = right_val {
                                if let Some(dt) = diff_type {
                                    ui.colored_label(dt.color(), val);
                                } else {
                                    ui.label(val);
                                }
                            } else {
                                ui.colored_label(egui::Color32::DARK_GRAY, "—");
                            }

                            if let Some(dt) = diff_type {
                                ui.colored_label(dt.color(), dt.label());
                            } else {
                                ui.label("✓");
                            }

                            ui.end_row();
                        }
                    });
            });
        });

    // 应用操作
    if should_close {
        compare.active = false;
    }
    if toggle_show_only_diff {
        compare.show_only_diff = !compare.show_only_diff;
    }
    if let Some(file) = load_left_file {
        let _ = compare.load_left(file);
    }
    if let Some(file) = load_right_file {
        let _ = compare.load_right(file);
    }
    if load_left_current {
        if let Some(path) = app.current_path().cloned() {
            let _ = compare.load_left(path);
        }
    }
    if load_right_current {
        if let Some(path) = app.current_path().cloned() {
            let _ = compare.load_right(path);
        }
    }
}
