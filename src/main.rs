use photo_exit::PhotoExitApp;

fn main() -> eframe::Result<()> {
    // 配置中文字体支持
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("Photo EXIF Tool"),
        ..Default::default()
    };

    eframe::run_native(
        "Photo EXIF Tool",
        options,
        Box::new(|cc| {
            // 加载中文字体
            setup_chinese_fonts(&cc.egui_ctx);
            Box::new(PhotoExitApp::new(cc))
        }),
    )
}

/// 配置中文字体以支持中文显示
fn setup_chinese_fonts(ctx: &eframe::egui::Context) {
    let mut fonts = eframe::egui::FontDefinitions::default();

    // 尝试加载微软雅黑字体（Windows）
    let font_paths = [
        ("Microsoft YaHei", "C:\\Windows\\Fonts\\msyh.ttc"),
        ("SimHei", "C:\\Windows\\Fonts\\simhei.ttf"),
        ("SimSun", "C:\\Windows\\Fonts\\simsun.ttc"),
        // macOS
        ("PingFang SC", "/System/Library/Fonts/PingFang.ttc"),
        ("Hiragino Sans GB", "/Library/Fonts/Songti.ttc"),
        // Linux
        ("Noto Sans CJK SC", "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
        ("Noto Sans CJK SC", "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc"),
        ("WenQuanYi Micro Hei", "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc"),
        ("Droid Sans Fallback", "/usr/share/fonts/truetype/droid/DroidSansFallback.ttf"),
    ];

    for (name, path) in font_paths.iter() {
        if std::path::Path::new(path).exists() {
            if let Ok(font_data) = std::fs::read(path) {
                fonts.font_data.insert(
                    name.to_string(),
                    eframe::egui::FontData::from_owned(font_data).into(),
                );

                // 将中文字体插入到字体族的前面
                fonts.families
                    .entry(eframe::egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, name.to_string());
                fonts.families
                    .entry(eframe::egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, name.to_string());

                eprintln!("✅ 已加载中文字体: {} ({})", name, path);
                break;
            }
        }
    }

    ctx.set_fonts(fonts);
}
