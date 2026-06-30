use crate::model::{AppState, ExifTag, ExifValue};
use crate::exif::{GpxMatcher, ExifWriter};
use anyhow::Result;
use std::path::PathBuf;

/// GPX 写入窗口状态
#[derive(Debug)]
pub struct GpxWindowState {
    pub gpx_path: Option<PathBuf>,
    pub matcher: Option<GpxMatcher>,
    pub preview_results: Vec<GpxPreviewResult>,
    pub status_message: String,
    pub active: bool,
}

impl Default for GpxWindowState {
    fn default() -> Self {
        Self {
            gpx_path: None,
            matcher: None,
            preview_results: Vec::new(),
            status_message: String::new(),
            active: false,
        }
    }
}

/// 单张照片的 GPX 匹配预览
#[derive(Debug, Clone)]
pub struct GpxPreviewResult {
    pub file_name: String,
    pub photo_time: Option<String>,
    pub matched: bool,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub ele: Option<f64>,
}

impl GpxWindowState {
    /// 加载 GPX 文件
    pub fn load_gpx(&mut self, path: PathBuf) -> Result<()> {
        let matcher = GpxMatcher::from_file(&path)?;
        let count = matcher.point_count();
        let range = matcher.time_range();
        self.gpx_path = Some(path);
        self.matcher = Some(matcher);

        if let Some((start, end)) = range {
            self.status_message = format!(
                "已加载 GPX: {} 个轨迹点，时间范围 {} ~ {}",
                count,
                start.format("%Y-%m-%d %H:%M:%S"),
                end.format("%Y-%m-%d %H:%M:%S")
            );
        } else {
            self.status_message = format!("已加载 GPX: {} 个轨迹点", count);
        }

        Ok(())
    }

    /// 预览所有照片的匹配结果
    pub fn preview_all(&mut self, app: &AppState) {
        self.preview_results.clear();

        if let Some(matcher) = &self.matcher {
            for path in &app.file_paths {
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let photo_time = app
                    .exif_entries
                    .iter()
                    .find(|(t, _)| t.id == 0x9003)
                    .map(|(_, v)| v.to_display_string());

                let result = if let Some(ref time_str) = photo_time {
                    if let Some((lat_dms, _lat_ref, lon_dms, _lon_ref, ele)) =
                        matcher.match_photo(time_str)
                    {
                        let lat = lat_dms.0 as f64
                            + lat_dms.1 as f64 / 60.0
                            + lat_dms.2 as f64 / 3600.0;
                        let lon = lon_dms.0 as f64
                            + lon_dms.1 as f64 / 60.0
                            + lon_dms.2 as f64 / 3600.0;
                        GpxPreviewResult {
                            file_name,
                            photo_time: Some(time_str.clone()),
                            matched: true,
                            lat: Some(lat),
                            lon: Some(lon),
                            ele,
                        }
                    } else {
                        GpxPreviewResult {
                            file_name,
                            photo_time: Some(time_str.clone()),
                            matched: false,
                            lat: None,
                            lon: None,
                            ele: None,
                        }
                    }
                } else {
                    GpxPreviewResult {
                        file_name,
                        photo_time: None,
                        matched: false,
                        lat: None,
                        lon: None,
                        ele: None,
                    }
                };

                self.preview_results.push(result);
            }
        }
    }

    /// 对当前图片写入 GPS（带 undo 记录）
    pub fn write_to_current(&self, app: &mut AppState) -> Result<()> {
        let matcher = self
            .matcher
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("未加载 GPX 文件"))?;

        let path = app
            .current_path()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("没有打开文件"))?;

        let photo_time = app
            .exif_entries
            .iter()
            .find(|(t, _)| t.id == 0x9003)
            .map(|(_, v)| v.to_display_string())
            .ok_or_else(|| anyhow::anyhow!("当前图片没有拍摄时间"))?;

        let match_result = matcher
            .match_photo(&photo_time)
            .ok_or_else(|| anyhow::anyhow!("无法匹配 GPS 轨迹点（时间差 > 5分钟）"))?;

        let (lat_dms, lat_ref, lon_dms, lon_ref, ele) = match_result;

        // 记录原始 GPS 值用于 undo
        let gps_tags = [
            (0x0001, "GPS", "纬度方向"),
            (0x0002, "GPS", "纬度"),
            (0x0003, "GPS", "经度方向"),
            (0x0004, "GPS", "经度"),
            (0x0005, "GPS", "高度参考"),
            (0x0006, "GPS", "高度"),
            (0x0007, "GPS", "GPS时间"),
        ];
        let mut old_entries = Vec::new();
        for (id, ifd, name) in &gps_tags {
            let tag = ExifTag::new(*id, *ifd, *name);
            if let Some(old) = app.exif_entries.get(&tag) {
                old_entries.push((tag.clone(), Some(old.clone())));
            } else {
                old_entries.push((tag.clone(), None));
            }
        }

        app.exif_entries.insert(
            ExifTag::new(0x0001, "GPS", "纬度方向"),
            ExifValue::Ascii(lat_ref.to_string()),
        );
        app.exif_entries.insert(
            ExifTag::new(0x0002, "GPS", "纬度"),
            ExifValue::Rational(lat_dms.0 * 1000000, 1000000),
        );
        app.exif_entries.insert(
            ExifTag::new(0x0003, "GPS", "经度方向"),
            ExifValue::Ascii(lon_ref.to_string()),
        );
        app.exif_entries.insert(
            ExifTag::new(0x0004, "GPS", "经度"),
            ExifValue::Rational(lon_dms.0 * 1000000, 1000000),
        );

        if let Some(elevation) = ele {
            app.exif_entries.insert(
                ExifTag::new(0x0005, "GPS", "高度参考"),
                ExifValue::Byte(if elevation < 0.0 { vec![1] } else { vec![0] }),
            );
            app.exif_entries.insert(
                ExifTag::new(0x0006, "GPS", "高度"),
                ExifValue::Rational((elevation.abs() * 100.0) as u32, 100),
            );
        }

        app.exif_entries.insert(
            ExifTag::new(0x0007, "GPS", "GPS时间"),
            ExifValue::Ascii(photo_time.clone()),
        );

        // 记录 undo：使用批量标签标记
        for (tag, old) in &old_entries {
            let new_val = app.exif_entries.get(tag).cloned().unwrap_or(ExifValue::Ascii(String::new()));
            app.push_undo(tag.clone(), old.clone().unwrap_or(ExifValue::Ascii(String::new())), new_val);
        }

        ExifWriter::write(&path, &app.exif_entries)?;
        app.original_exif = app.exif_entries.clone();
        app.set_status(
            format!(
                "GPS 已写入: {:.6}°{} {:.6}°{}",
                lat_dms.0 as f64 + lat_dms.1 as f64 / 60.0 + lat_dms.2 as f64 / 3600.0,
                lat_ref,
                lon_dms.0 as f64 + lon_dms.1 as f64 / 60.0 + lon_dms.2 as f64 / 3600.0,
                lon_ref
            ),
            crate::model::StatusLevel::Success,
        );

        Ok(())
    }

    /// 批量写入所有匹配的照片
    pub fn write_to_all(&self, app: &mut AppState) -> Result<usize> {
        let matcher = self
            .matcher
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("未加载 GPX 文件"))?;

        let mut success_count = 0;

        for path in &app.file_paths {
            let (entries, _) = match crate::exif::ExifReader::read(path) {
                Ok(e) => e,
                Err(_) => continue,
            };

            let photo_time = entries
                .iter()
                .find(|(t, _)| t.id == 0x9003)
                .map(|(_, v)| v.to_display_string());

            let time_str = match photo_time {
                Some(t) => t,
                None => continue,
            };

            let match_result = match matcher.match_photo(&time_str) {
                Some(r) => r,
                None => continue,
            };

            let (lat_dms, lat_ref, lon_dms, lon_ref, ele) = match_result;
            let mut new_entries = entries;

            new_entries.insert(
                ExifTag::new(0x0001, "GPS", "纬度方向"),
                ExifValue::Ascii(lat_ref.to_string()),
            );
            new_entries.insert(
                ExifTag::new(0x0002, "GPS", "纬度"),
                ExifValue::Rational(lat_dms.0 * 1000000, 1000000),
            );
            new_entries.insert(
                ExifTag::new(0x0003, "GPS", "经度方向"),
                ExifValue::Ascii(lon_ref.to_string()),
            );
            new_entries.insert(
                ExifTag::new(0x0004, "GPS", "经度"),
                ExifValue::Rational(lon_dms.0 * 1000000, 1000000),
            );

            if let Some(elevation) = ele {
                new_entries.insert(
                    ExifTag::new(0x0005, "GPS", "高度参考"),
                    ExifValue::Byte(if elevation < 0.0 { vec![1] } else { vec![0] }),
                );
                new_entries.insert(
                    ExifTag::new(0x0006, "GPS", "高度"),
                    ExifValue::Rational((elevation.abs() * 100.0) as u32, 100),
                );
            }

            if ExifWriter::write(path, &new_entries).is_ok() {
                success_count += 1;
            }
        }

        // 更新当前图片状态
        if let Some(current) = app.current_path() {
            let (entries, _) = crate::exif::ExifReader::read(current).unwrap_or_default();
            app.exif_entries = entries.clone();
            app.original_exif = entries;
        }

        Ok(success_count)
    }
}

/// 渲染 GPX GPS 写入窗口
pub fn render_gpx_window(
    ui: &mut egui::Ui,
    gpx_state: &mut GpxWindowState,
    app: &mut AppState,
) {
    if !gpx_state.active {
        return;
    }

    // 预提取数据
    let gpx_file_name = gpx_state
        .gpx_path
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string());
    let status_msg = gpx_state.status_message.clone();
    let preview_count = gpx_state.preview_results.len();
    let matched_count = gpx_state.preview_results.iter().filter(|r| r.matched).count();
    let preview_clone = gpx_state.preview_results.clone();

    let mut pick_gpx: Option<PathBuf> = None;
    let mut do_refresh = false;
    let mut do_write_current = false;
    let mut do_write_all = false;
    let mut should_close = false;

    egui::Window::new("GPX GPS 写入")
        .open(&mut should_close)
        .default_width(700.0)
        .default_height(500.0)
        .resizable(true)
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label("GPX 轨迹文件:");
                if let Some(ref name) = gpx_file_name {
                    ui.label(name);
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "(未加载)");
                }
                if ui.button("📁 选择 GPX...").clicked() {
                    pick_gpx = rfd::FileDialog::new()
                        .add_filter("GPX", &["gpx"])
                        .pick_file();
                }
            });

            if !status_msg.is_empty() {
                ui.colored_label(egui::Color32::LIGHT_BLUE, &status_msg);
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("🔄 刷新预览").clicked() {
                    do_refresh = true;
                }
                if ui.button("✅ 写入当前图片").clicked() {
                    do_write_current = true;
                }
                if ui.button("✅✅ 批量写入所有").clicked() {
                    do_write_all = true;
                }
            });

            ui.separator();

            if preview_count == 0 {
                ui.centered_and_justified(|ui| {
                    ui.label("加载 GPX 文件后显示匹配预览");
                });
                return;
            }

            ui.label(format!("匹配结果: {}/{} 张照片", matched_count, preview_count));
            ui.add_space(4.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("gpx_preview_grid")
                    .num_columns(5)
                    .spacing([10.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.heading("文件名");
                        ui.heading("拍摄时间");
                        ui.heading("纬度");
                        ui.heading("经度");
                        ui.heading("状态");
                        ui.end_row();

                        for result in &preview_clone {
                            ui.label(&result.file_name);
                            ui.label(result.photo_time.as_deref().unwrap_or("(无时间)"));

                            if result.matched {
                                if let Some(lat) = result.lat {
                                    ui.colored_label(egui::Color32::GREEN, format!("{:.6}", lat));
                                } else {
                                    ui.label("—");
                                }
                                if let Some(lon) = result.lon {
                                    ui.colored_label(egui::Color32::GREEN, format!("{:.6}", lon));
                                } else {
                                    ui.label("—");
                                }
                                ui.colored_label(egui::Color32::GREEN, "✅ 匹配");
                            } else {
                                ui.colored_label(egui::Color32::DARK_GRAY, "—");
                                ui.colored_label(egui::Color32::DARK_GRAY, "—");
                                ui.colored_label(egui::Color32::YELLOW, "❌ 未匹配");
                            }
                            ui.end_row();
                        }
                    });
            });
        });

    // 应用操作
    if should_close {
        gpx_state.active = false;
    }
    if let Some(file) = pick_gpx {
        match gpx_state.load_gpx(file) {
            Ok(_) => {
                gpx_state.preview_all(app);
            }
            Err(e) => {
                gpx_state.status_message = format!("加载失败: {}", e);
            }
        }
    }
    if do_refresh {
        gpx_state.preview_all(app);
    }
    if do_write_current {
        match gpx_state.write_to_current(app) {
            Ok(_) => {
                gpx_state.status_message = "GPS 数据已写入当前图片".to_string();
            }
            Err(e) => {
                gpx_state.status_message = format!("写入失败: {}", e);
            }
        }
    }
    if do_write_all {
        match gpx_state.write_to_all(app) {
            Ok(count) => {
                gpx_state.status_message = format!("成功写入 {} 张照片", count);
                gpx_state.preview_all(app);
            }
            Err(e) => {
                gpx_state.status_message = format!("批量写入失败: {}", e);
            }
        }
    }
}
