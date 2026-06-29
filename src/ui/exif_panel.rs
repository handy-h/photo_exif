use crate::model::{AppState, ExifTag, ExifValue};
use crate::exif::ExifFormatter;

/// 渲染右侧 EXIF 信息面板
pub fn render_exif_panel(app: &mut AppState, ctx: &egui::Context) {
    if app.is_fullscreen {
        return;
    }

    // 写入确认对话框
    if app.pending_save {
        render_save_confirmation(app, ctx);
    }

    // 新增字段弹窗
    if app.show_add_tag_popup {
        render_add_tag_popup(app, ctx);
    }

    egui::SidePanel::right("exif_panel")
        .resizable(true)
        .default_width(450.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                render_extension_warning(app, ui);
                render_privacy_warning(app, ui);
                render_search_bar(app, ui);
                ui.separator();

                // 快捷编辑面板切换
                ui.horizontal(|ui| {
                    if ui.button("📋 快捷编辑").clicked() {
                        app.show_quick_edit = !app.show_quick_edit;
                    }
                });

                if app.show_quick_edit {
                    render_quick_edit_panel(app, ui);
                    ui.separator();
                }

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
    let mismatch = match &app.extension_warning {
        Some(m) => m.clone(),
        None => return,
    };
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

fn render_privacy_warning(app: &mut AppState, ui: &mut egui::Ui) {
    let risks = app.has_privacy_risk();
    if !risks.is_empty() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::RED, "🔒");
                ui.colored_label(
                    egui::Color32::RED,
                    format!("检测到 {} 个隐私相关字段（GPS/个人信息）", risks.len()),
                );
            });
            ui.horizontal(|ui| {
                if ui.button("一键清除隐私字段").clicked() {
                    for tag in &risks {
                        app.exif_entries.remove(tag);
                    }
                    app.set_status(
                        format!("已清除 {} 个隐私字段", risks.len()),
                        crate::model::StatusLevel::Success,
                    );
                }
            });
        });
        ui.add_space(4.0);
    }
}

fn render_search_bar(app: &mut AppState, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("🔍");
        ui.text_edit_singleline(&mut app.search_query);
        if ui.button("✕").clicked() {
            app.search_query.clear();
        }
    });
}

fn render_quick_edit_panel(app: &mut AppState, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.heading("快捷编辑");

        // 获取常用字段的当前值
        let mut date_time = get_field_value(app, 0x9003, "ExifIFD");
        let mut iso = get_field_value(app, 0x8827, "ExifIFD");
        let mut aperture = get_field_value(app, 0x829D, "ExifIFD");
        let mut shutter = get_field_value(app, 0x829A, "ExifIFD");
        let mut focal = get_field_value(app, 0x920A, "ExifIFD");
        let mut gps_lat = get_field_value(app, 0x0002, "GPS");
        let mut gps_lon = get_field_value(app, 0x0004, "GPS");

        egui::Grid::new("quick_edit_grid")
            .num_columns(2)
            .spacing([10.0, 6.0])
            .show(ui, |ui| {
                ui.label("拍摄时间:");
                if ui.text_edit_singleline(&mut date_time).changed() {
                    set_field_value(app, 0x9003, "ExifIFD", "拍摄时间", &date_time);
                }
                ui.end_row();

                ui.label("ISO:");
                if ui.text_edit_singleline(&mut iso).changed() {
                    set_field_value(app, 0x8827, "ExifIFD", "ISO", &iso);
                }
                ui.end_row();

                ui.label("光圈值:");
                if ui.text_edit_singleline(&mut aperture).changed() {
                    set_field_value(app, 0x829D, "ExifIFD", "光圈值", &aperture);
                }
                ui.end_row();

                ui.label("快门速度:");
                if ui.text_edit_singleline(&mut shutter).changed() {
                    set_field_value(app, 0x829A, "ExifIFD", "曝光时间", &shutter);
                }
                ui.end_row();

                ui.label("焦距:");
                if ui.text_edit_singleline(&mut focal).changed() {
                    set_field_value(app, 0x920A, "ExifIFD", "焦距", &focal);
                }
                ui.end_row();

                ui.label("GPS 纬度:");
                if ui.text_edit_singleline(&mut gps_lat).changed() {
                    set_field_value(app, 0x0002, "GPS", "纬度", &gps_lat);
                }
                ui.end_row();

                ui.label("GPS 经度:");
                if ui.text_edit_singleline(&mut gps_lon).changed() {
                    set_field_value(app, 0x0004, "GPS", "经度", &gps_lon);
                }
                ui.end_row();
            });
    });
}

fn get_field_value(app: &AppState, id: u16, ifd: &str) -> String {
    app.exif_entries
        .iter()
        .find(|(tag, _)| tag.id == id && tag.ifd == ifd)
        .map(|(_, v)| v.to_display_string())
        .unwrap_or_default()
}

fn set_field_value(app: &mut AppState, id: u16, ifd: &str, name: &str, text: &str) {
    if text.is_empty() {
        return;
    }
    let tag = ExifTag::new(id, ifd, name);
    let new_value = ExifValue::from_string(text);
    if let Some(old_value) = app.exif_entries.get(&tag) {
        app.push_undo(tag.clone(), old_value.clone(), new_value.clone());
    } else {
        app.push_undo(tag.clone(), ExifValue::Ascii(String::new()), new_value.clone());
    }
    app.exif_entries.insert(tag, new_value);
}

fn render_exif_table(app: &mut AppState, ui: &mut egui::Ui) {
    let grouped = group_exif_entries(&app.exif_entries);

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

        let is_expanded = *app
            .expanded_groups
            .entry(group)
            .or_insert(group.default_expanded());

        egui::CollapsingHeader::new(group.label())
            .default_open(is_expanded)
            .show(ui, |ui| {
                for (tag, value) in &filtered {
                    render_exif_row(app, ui, tag, value);
                }
            });
    }
}

fn render_exif_row(app: &mut AppState, ui: &mut egui::Ui, tag: &ExifTag, value: &ExifValue) {
    // 隐私风险高亮
    let is_privacy = tag.ifd == "GPS"
        || tag.id == 0x8298
        || tag.id == 0x927C
        || tag.id == 0x9286
        || tag.id == 0xA435;

    let is_selected = app.selected_tags.contains(tag);
    let is_editing = app.editing_tag.as_ref() == Some(tag);

    ui.push_id(tag, |ui| {
        let row_response = ui.horizontal(|ui| {
            // 选择框
            let mut checked = is_selected;
            if ui.checkbox(&mut checked, "").changed() {
                if checked && !is_selected {
                    app.selected_tags.push(tag.clone());
                } else if !checked && is_selected {
                    app.selected_tags.retain(|t| t != tag);
                }
            }

            // Tag 名（隐私字段红色高亮）
            if is_privacy {
                ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("{}:", tag.name));
            } else {
                ui.label(format!("{}:", tag.name));
            }

            // 值 - 可编辑
            let display_value = ExifFormatter::format(tag.id, value);

            if is_editing {
                let mut text = value.to_display_string();
                let response = ui.text_edit_singleline(&mut text);
                if response.lost_focus() {
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
                // 双击进入编辑
                let label_response = ui.add(egui::Label::new(&display_value).wrap(true));
                if label_response.double_clicked() {
                    app.editing_tag = Some(tag.clone());
                }
            }
        });

        // 右键菜单
        row_response.response.context_menu(|ui| {
            if ui.button("✏️ 修改").clicked() {
                app.editing_tag = Some(tag.clone());
                ui.close_menu();
            }
            if ui.button("🗑️ 删除").clicked() {
                app.selected_tags.clear();
                app.selected_tags.push(tag.clone());
                crate::io::FileOps::delete_selected(app);
                ui.close_menu();
            }
            ui.separator();
            if ui.button("➕ 新增字段").clicked() {
                app.context_menu_tag = Some(tag.clone());
                app.show_add_tag_popup = true;
                app.new_tag_id = String::new();
                app.new_tag_ifd = String::new();
                app.new_tag_name = String::new();
                app.new_tag_value = String::new();
                ui.close_menu();
            }
        });
    });
}

fn render_save_confirmation(app: &mut AppState, ctx: &egui::Context) {
    let changes = app.get_changes();

    egui::Window::new("确认保存")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            if changes.is_empty() {
                ui.label("没有需要保存的更改");
                if ui.button("关闭").clicked() {
                    app.pending_save = false;
                }
                return;
            }

            ui.label(format!("即将修改 {} 个字段：", changes.len()));
            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    egui::Grid::new("changes_grid")
                        .num_columns(3)
                        .spacing([10.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.heading("字段");
                            ui.heading("原值");
                            ui.heading("新值");
                            ui.end_row();

                            for (tag, old, new) in &changes {
                                ui.label(&tag.name);
                                ui.label(
                                    old.as_ref()
                                        .map(|v| v.to_display_string())
                                        .unwrap_or_else(|| "(新增)".into()),
                                );
                                ui.label(
                                    new.as_ref()
                                        .map(|v| v.to_display_string())
                                        .unwrap_or_else(|| "(删除)".into()),
                                );
                                ui.end_row();
                            }
                        });
                });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("✅ 确认保存").clicked() {
                    let result = crate::io::FileOps::do_save(app);
                    if let Err(e) = result {
                        app.set_status(format!("保存失败: {}", e), crate::model::StatusLevel::Error);
                    }
                    app.pending_save = false;
                }
                if ui.button("❌ 取消").clicked() {
                    app.pending_save = false;
                }
            });
        });
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
        (_, 0x829A) | (_, 0x829D) | (_, 0x9204) | (_, 0x8822) | (_, 0x9207)
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

fn render_add_tag_popup(app: &mut AppState, ctx: &egui::Context) {
    let mut open = app.show_add_tag_popup;

    egui::Window::new("新增 EXIF 字段")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label("输入新字段的信息：");
            ui.add_space(8.0);

            egui::Grid::new("add_tag_grid")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Tag ID (十六进制):");
                    ui.text_edit_singleline(&mut app.new_tag_id);
                    ui.end_row();

                    ui.label("IFD:");
                    ui.text_edit_singleline(&mut app.new_tag_ifd);
                    ui.end_row();

                    ui.label("名称:");
                    ui.text_edit_singleline(&mut app.new_tag_name);
                    ui.end_row();

                    ui.label("值:");
                    ui.text_edit_singleline(&mut app.new_tag_value);
                    ui.end_row();
                });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("✅ 确认添加").clicked() {
                    let id_str = app.new_tag_id.trim().to_string();
                    let ifd = app.new_tag_ifd.trim().to_string();
                    let name = app.new_tag_name.trim().to_string();
                    let value = app.new_tag_value.trim().to_string();

                    if id_str.is_empty() || ifd.is_empty() || name.is_empty() || value.is_empty() {
                        app.set_status("请填写所有字段", crate::model::StatusLevel::Warning);
                    } else {
                        let id = match u16::from_str_radix(id_str.trim_start_matches("0x"), 16) {
                            Ok(v) => v,
                            Err(_) => {
                                app.set_status("Tag ID 格式错误，请输入十六进制（如 0x9003）", crate::model::StatusLevel::Error);
                                return;
                            }
                        };

                        let tag = crate::model::ExifTag::new(id, &ifd, &name);
                        let new_value = crate::model::ExifValue::from_string(&value);

                        if let Some(old_value) = app.exif_entries.get(&tag) {
                            app.push_undo(tag.clone(), old_value.clone(), new_value.clone());
                        } else {
                            app.push_undo(tag.clone(), crate::model::ExifValue::Ascii(String::new()), new_value.clone());
                        }

                        app.exif_entries.insert(tag.clone(), new_value);
                        app.set_status(
                            format!("已新增字段: {} ({})", name, ifd),
                            crate::model::StatusLevel::Success,
                        );
                    }

                    app.show_add_tag_popup = false;
                }
                if ui.button("❌ 取消").clicked() {
                    app.show_add_tag_popup = false;
                }
            });
        });

    if !open {
        app.show_add_tag_popup = false;
    }
}
