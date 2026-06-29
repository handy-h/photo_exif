use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// ExifGroup - EXIF 字段分组
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ExifGroup {
    CameraInfo,
    Exposure,
    GPS,
    Lens,
    Thumbnail,
    Other,
}

impl ExifGroup {
    pub fn label(self) -> &'static str {
        match self {
            ExifGroup::CameraInfo => "相机信息",
            ExifGroup::Exposure => "曝光参数",
            ExifGroup::GPS => "GPS",
            ExifGroup::Lens => "镜头信息",
            ExifGroup::Thumbnail => "缩略图",
            ExifGroup::Other => "其他",
        }
    }

    pub fn default_expanded(self) -> bool {
        match self {
            ExifGroup::CameraInfo => true,
            ExifGroup::Exposure => true,
            ExifGroup::GPS => true,
            ExifGroup::Lens => true,
            ExifGroup::Thumbnail => false,
            ExifGroup::Other => false,
        }
    }
}

// ============================================================================
// ExifTag - EXIF 标签（tag id + 显示名）
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExifTag {
    pub id: u16,
    pub ifd: String,
    pub name: String,
}

impl ExifTag {
    pub fn new(id: u16, ifd: &str, name: &str) -> Self {
        Self {
            id,
            ifd: ifd.to_string(),
            name: name.to_string(),
        }
    }

    pub fn display_name(&self) -> String {
        format!("{} ({})", self.name, self.ifd)
    }
}

// ============================================================================
// ExifValue - EXIF 值的类型枚举
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ExifValue {
    Byte(Vec<u8>),
    Ascii(String),
    Short(u16),
    Long(u32),
    Rational(u32, u32),
    SRational(i32, i32),
    Undefined(Vec<u8>),
    Slice(Vec<u8>),
}

impl ExifValue {
    /// 将值格式化为可读字符串
    pub fn to_display_string(&self) -> String {
        match self {
            ExifValue::Byte(v) => format!("{:02X?}", v),
            ExifValue::Ascii(s) => s.trim_end_matches('\0').to_string(),
            ExifValue::Short(v) => v.to_string(),
            ExifValue::Long(v) => v.to_string(),
            ExifValue::Rational(n, d) => {
                if *d == 0 {
                    "0".to_string()
                } else if *n % *d == 0 {
                    format!("{}", n / d)
                } else {
                    format!("{}/{}", n, d)
                }
            }
            ExifValue::SRational(n, d) => {
                if *d == 0 {
                    "0".to_string()
                } else if *n % *d == 0 {
                    format!("{}", n / d)
                } else {
                    format!("{}/{}", n, d)
                }
            }
            ExifValue::Undefined(v) => format!("{:02X?}", v),
            ExifValue::Slice(v) => format!("{:02X?}", v),
        }
    }

    /// 从字符串解析回值类型
    pub fn from_string(s: &str) -> Self {
        // 尝试解析为 rational (如 "28/10" 或 "2.8")
        if s.contains('/') {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                if let (Ok(n), Ok(d)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    return ExifValue::Rational(n, d);
                }
                if let (Ok(n), Ok(d)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                    return ExifValue::SRational(n, d);
                }
            }
        }
        if let Ok(v) = s.parse::<u16>() {
            return ExifValue::Short(v);
        }
        if let Ok(v) = s.parse::<u32>() {
            return ExifValue::Long(v);
        }
        if let Ok(v) = s.parse::<f64>() {
            let scaled = (v * 10000.0).round() as u32;
            return ExifValue::Rational(scaled, 10000);
        }
        ExifValue::Ascii(s.to_string())
    }
}

// ============================================================================
// ExtensionMismatch - 扩展名校验结果
// ============================================================================

#[derive(Debug, Clone)]
pub struct ExtensionMismatch {
    pub actual_format: String,
    pub expected_ext: String,
    pub actual_ext: String,
}

// ============================================================================
// AppState - 应用全局状态
// ============================================================================

/// AppState - 应用全局状态
pub struct AppState {
    // 文件列表
    pub folder_path: Option<PathBuf>,
    pub file_paths: Vec<PathBuf>,
    pub current_index: usize,

    // 当前图片
    pub current_image: Option<egui::ColorImage>,
    pub image_size: Option<[u32; 2]>,

    // EXIF 数据
    pub exif_entries: HashMap<ExifTag, ExifValue>,
    pub original_exif: HashMap<ExifTag, ExifValue>,
    pub undo_stack: Vec<(ExifTag, ExifValue, ExifValue)>, // (tag, old, new)

    // UI 状态
    pub search_query: String,
    pub expanded_groups: HashMap<ExifGroup, bool>,
    pub selected_tags: Vec<ExifTag>,
    pub zoom: f32,
    pub is_fullscreen: bool,
    pub editing_tag: Option<ExifTag>,

    // 右键菜单
    pub context_menu_tag: Option<ExifTag>,

    // 新增字段弹窗
    pub show_add_tag_popup: bool,
    pub new_tag_id: String,
    pub new_tag_ifd: String,
    pub new_tag_name: String,
    pub new_tag_value: String,

    // 警告
    pub extension_warning: Option<ExtensionMismatch>,
    pub pending_rename: Option<PathBuf>,

    // 状态消息
    pub status_message: Option<(String, StatusLevel)>,

    // 批量操作
    pub clipboard_exif: Option<HashMap<ExifTag, ExifValue>>,

    // 写入确认对话框
    pub pending_save: bool,

    // 快捷编辑面板
    pub show_quick_edit: bool,

    // 缩略图画廊
    pub thumbnails: Vec<ThumbnailInfo>,
    pub thumbnail_scroll: f32,

    // 1:1 像素视图模式
    pub pixel_perfect: bool,

    // 鼠标是否在预览区域上方
    pub pointer_over_preview: bool,

    // 对比模式
    pub compare_state: crate::ui::compare::CompareState,

    // GPX 写入窗口
    pub gpx_window: crate::ui::gpx_window::GpxWindowState,

    // EXIF 修复窗口
    pub repair_window: crate::ui::repair_window::RepairWindowState,
}

#[derive(Debug, Clone)]
pub struct ThumbnailInfo {
    pub path: PathBuf,
    pub image: Option<egui::ColorImage>,
    pub is_loading: bool,
    pub texture_id: Option<egui::TextureId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            folder_path: None,
            file_paths: Vec::new(),
            current_index: 0,
            current_image: None,
            image_size: None,
            exif_entries: HashMap::new(),
            original_exif: HashMap::new(),
            undo_stack: Vec::new(),
            search_query: String::new(),
            expanded_groups: HashMap::from([
                (ExifGroup::CameraInfo, true),
                (ExifGroup::Exposure, true),
                (ExifGroup::GPS, true),
                (ExifGroup::Lens, true),
                (ExifGroup::Thumbnail, false),
                (ExifGroup::Other, false),
            ]),
            selected_tags: Vec::new(),
            zoom: 1.0,
            is_fullscreen: false,
            editing_tag: None,
            context_menu_tag: None,
            show_add_tag_popup: false,
            new_tag_id: String::new(),
            new_tag_ifd: String::new(),
            new_tag_name: String::new(),
            new_tag_value: String::new(),
            extension_warning: None,
            pending_rename: None,
            status_message: None,
            clipboard_exif: None,
            pending_save: false,
            show_quick_edit: false,
            thumbnails: Vec::new(),
            thumbnail_scroll: 0.0,
            pixel_perfect: false,
            pointer_over_preview: false,
            compare_state: Default::default(),
            gpx_window: Default::default(),
            repair_window: Default::default(),
        }
    }

    pub fn current_path(&self) -> Option<&PathBuf> {
        self.file_paths.get(self.current_index)
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.exif_entries != self.original_exif
    }

    pub fn push_undo(&mut self, tag: ExifTag, old: ExifValue, new: ExifValue) {
        self.undo_stack.push((tag, old, new));
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some((tag, old, _)) = self.undo_stack.pop() {
            self.exif_entries.insert(tag.clone(), old);
            true
        } else {
            false
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>, level: StatusLevel) {
        self.status_message = Some((msg.into(), level));
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// 检测当前 EXIF 是否有隐私风险（GPS / 个人信息）
    pub fn has_privacy_risk(&self) -> Vec<ExifTag> {
        self.exif_entries
            .keys()
            .filter(|tag| {
                // GPS 相关
                tag.ifd == "GPS"
                // 版权
                || tag.id == 0x8298
                // 制造商备注（可能含序列号）
                || tag.id == 0x927C
                // 用户注释
                || tag.id == 0x9286
                // 镜头序列号等
                || tag.id == 0xA435
            })
            .cloned()
            .collect()
    }

    /// 获取变更列表（用于写入确认）
    pub fn get_changes(&self) -> Vec<(ExifTag, Option<ExifValue>, Option<ExifValue>)> {
        let mut changes = Vec::new();

        // 修改和新增
        for (tag, value) in &self.exif_entries {
            if let Some(orig) = self.original_exif.get(tag) {
                if orig != value {
                    changes.push((tag.clone(), Some(orig.clone()), Some(value.clone())));
                }
            } else {
                changes.push((tag.clone(), None, Some(value.clone())));
            }
        }

        // 删除
        for (tag, value) in &self.original_exif {
            if !self.exif_entries.contains_key(tag) {
                changes.push((tag.clone(), Some(value.clone()), None));
            }
        }

        changes
    }
}
