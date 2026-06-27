use crate::model::{AppState, ExifEntry, ExifGroup, ExifTag, ExifValue};
use crate::exif::ExifFormatter;

/// 渲染右侧 EXIF 信息面板
pub fn render_exif_panel(app: &mut AppState, ctx: &egui::Context) {
    if app.is_fullscreen {
        return;
    }

    egui::SidePanel::right("exif_panel")
        .resizable(true)
        .default_width(450.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                render_extension_warning(app, ui);
                render_search_bar(app, ui);
                ui.separator();

                if app.exif_entries.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("此图片没有 EXIF 信息");
                    });
                    return;
                }

                render_exif_table(app, ui);
            });
        });
}

fn render_extension_warning(app: &mut AppState, ui: &mut egui::Ui) {
    if let Some(mismatch) = &app.extension_warning {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::YELLOW, "⚠");
                ui.label(format!(
                    "实际格式: {}, 扩展名: {}",
                    mismatch.actual_format, mismatch.actual_ext
                ));
            });

            ui.horizontal(|ui| {
                if ui.button("修正扩展名").clicked() {
                    let _ = crate::io::FileOps::fix_extension(app);
                }
                if ui.button("忽略").clicked() {
                    app.extension_warning = None;
                }
            });
        });
        ui.add_space(4.0);
    }
}

fn render_search_bar(app: &mut AppState, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("🔍");
        let response = ui.text_edit_singleline(&mut app.search_query);
        if response.changed() && app.search_query.is_empty() {
            // 搜索清空，显示所有
        }
        if ui.button("✕").clicked() {
            app.search_query.clear();
        }
    });
}

fn render_exif_table(app: &mut AppState, ui: &mut egui::Ui) {
    // 获取分组后的条目
    let grouped = group_exif_entries(&app.exif_entries);

    // 过滤
    let search = app.search_query.to_lowercase();

    for (group, entries) in grouped {
        // 过滤搜索
        let filtered: Vec<_> = if search.is_empty() {
            entries
        } else {
            entries
                .into_iter()
                .filter(|(tag, _)| {
                    tag.name.to_lowercase().contains(&search)
                        || tag.ifd.to_lowercase().contains(&search)
                })
                .collect()
        };

        if filtered.is_empty() {
            continue;
        }

        ui.group(|ui| {
            let is_expanded = app.expanded_groups.entry(group).or_insert(true);
            let header = egui::CollapsingHeader::new(group.label())
                .default_open(*is_expanded)
                .show(ui, |ui| {
                    for (tag, value) in &filtered {
                        render_exif_row(app, ui, tag, value);
                    }
                });

            // 更新展开状态
            if header.inner {
                *is_expanded = true;
            } else if header.fully_open() || !header.fully_closed() {
                // 检测是否被用户切换
            }
        });
    }
}

fn render_exif_row(app: &mut AppState, ui: &mut egui::Ui, tag: &ExifTag, value: &ExifValue) {
    ui.horizontal(|ui| {
        // 选择框
        let is_selected = app.selected_tags.contains(tag);
        if ui.checkbox(&mut is_selected.then(|| true).unwrap_or(false), "").changed() {
            if is_selected {
                app.selected_tags.retain(|t| t != tag);
            } else {
                app.selected_tags.push(tag.clone());
            }
        }

        // Tag 名
        ui.label(format!("{}:", tag.name));

        // 值 - 可编辑
        let display_value = ExifFormatter::format(tag.id, value);
        let is_editing = app.editing_tag.as_ref() == Some(tag);

        if is_editing {
            let mut text = value.to_display_string();
            let response = ui.text_edit_singleline(&mut text);
            if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !text.is_empty() {
                    let old_value = value.clone();
                    let new_value = ExifValue::from_string(&text);
                    app.push_undo(tag.clone(), old_value, new_value.clone());
                    app.exif_entries.insert(tag.clone(), new_value);
                }
                app.editing_tag = None;
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.editing_tag = None;
            }
        } else {
            ui.label(&display_value);
        }
    });

    // 双击编辑
    let row_rect = ui.max_rect();
    if ui.input(|i| i.pointer.is_missing()) {
        // 处理双击
    }
}

fn group_exif_entries(
    entries: &std::collections::HashMap<crate::model::ExifTag, crate::model::ExifValue>,
) -> std::collections::BTreeMap<crate::model::ExifGroup, Vec<(crate::model::ExifTag, crate::model::ExifValue)>> {
    let mut groups: std::collections::BTreeMap<crate::model::ExifGroup, Vec<(crate::model::ExifTag, crate::model::ExifValue)>> =
        std::collections::BTreeMap::new();

    for (tag, value) in entries {
        let group = tag_to_group(tag);
        groups.entry(group).or_default().push((tag.clone(), value.clone()));
    }

    groups
}

fn tag_to_group(tag: &crate::model::ExifTag) -> crate::model::ExifGroup {
    use crate::model::ExifGroup;

    let id = tag.id;
    let ifd = tag.ifd.as_str();

    match (ifd, id) {
        // GPS
        ("GPS", _) => ExifGroup::GPS,
        // Thumbnail
        ("Thumbnail", _) => ExifGroup::Thumbnail,
        // 曝光参数
        (_, 0x829A) | (_, 0x829D) | (_, 0x8827) | (_, 0x9204) | (_, 0x8822) | (_, 0x9207)
        | (_, 0x9208) | (_, 0x9209) | (_, 0x9214) | (_, 0xA405) | (_, 0xA406) => ExifGroup::Exposure,
        // 镜头信息
        (_, 0xA432) | (_, 0xA433) | (_, 0xA434) | (_, 0xA435) | (_, 0xA436) | (_, 0xA437) => {
            ExifGroup::Lens
        }
        // 相机信息
        (_, 0x010F) | (_, 0x0110) | (_, 0x0112) | (_, 0x0131) | (_, 0x0132) | (_, 0x013B)
        | (_, 0x8298) | (_, 0x8769) => ExifGroup::CameraInfo,
        // 其他
        _ => ExifGroup::Other,
    }
}
