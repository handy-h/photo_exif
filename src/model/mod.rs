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
        matches!(self, ExifGroup::CameraInfo | ExifGroup::Exposure | ExifGroup::GPS | ExifGroup::Lens)
    }
}

// ============================================================================
// ExifTag - EXIF 标签标识
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
}

// ============================================================================
// ExifValue - EXIF 值类型
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
    /// 转换为显示字符串
    pub fn to_display_string(&self) -> String {
        match self {
            ExifValue::Byte(v) => format!("{:02X?}", v),
            ExifValue::Ascii(v) => v.clone(),
            ExifValue::Short(v) => v.to_string(),
            ExifValue::Long(v) => v.to_string(),
            ExifValue::Rational(n, d) => {
                if *d == 0 {
                    format!("{}/{}", n, d)
                } else if *d == 1 {
                    n.to_string()
                } else if *n % *d == 0 {
                    format!("{}", n / d)
                } else {
                    let f = *n as f64 / *d as f64;
                    format!("{:.2}", f)
                }
            }
            ExifValue::SRational(n, d) => {
                if *d == 0 {
                    format!("{}/{}", n, d)
                } else if *d == 1 {
                    n.to_string()
                } else {
                    let f = *n as f64 / *d as f64;
                    format!("{:.2}", f)
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
        ExifValue::Ascii(s.to_string())
    }
}

// ============================================================================
// ExtensionMismatch - 扩展名不匹配警告
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionMismatch {
    pub actual_format: String,
    pub extension: String,
}

impl ExtensionMismatch {
    pub fn new(actual_format: String, extension: String) -> Self {
        Self {
            actual_format,
            extension,
        }
    }
}

// ============================================================================
// AppState - 应用全局状态
// ============================================================================

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

    // 纹理缓存（避免每帧重复加载）
    pub current_texture: Option<egui::TextureHandle>,

    // 鼠标是否在预览区域上方
    pub pointer_over_preview: bool,

    // 最近打开菜单
    pub show_recent_menu: bool,

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
            current_texture: None,
            pointer_over_preview: false,
            show_recent_menu: false,
            compare_state: Default::default(),
            gpx_window: Default::default(),
            repair_window: Default::default(),
        }
    }

    /// 检查是否有隐私风险字段
    pub fn has_privacy_risk(&self) -> Vec<String> {
        let mut risks = Vec::new();
        for (tag, _) in &self.exif_entries {
            let name_lower = tag.name.to_lowercase();
            if name_lower.contains("gps") || name_lower.contains("location") {
                risks.push(format!("GPS: {}", tag.name));
            }
            if name_lower.contains("serial") || name_lower.contains("序列号") {
                risks.push(format!("序列号: {}", tag.name));
            }
        }
        risks
    }

    /// 获取当前更改列表
    pub fn get_changes(&self) -> Vec<(ExifTag, ExifValue, ExifValue)> {
        let mut changes = Vec::new();
        for (tag, new_val) in &self.exif_entries {
            if let Some(old_val) = self.original_exif.get(tag) {
                if old_val != new_val {
                    changes.push((tag.clone(), old_val.clone(), new_val.clone()));
                }
            } else {
                changes.push((tag.clone(), ExifValue::Ascii(String::new()), new_val.clone()));
            }
        }
        // 检查被删除的字段
        for (tag, old_val) in &self.original_exif {
            if !self.exif_entries.contains_key(tag) {
                changes.push((tag.clone(), old_val.clone(), ExifValue::Ascii(String::new())));
            }
        }
        changes
    }

    pub fn current_path(&self) -> Option<&PathBuf> {
        self.file_paths.get(self.current_index)
    }

    /// 检查是否有未保存的更改
    pub fn has_unsaved_changes(&self) -> bool {
        self.exif_entries != self.original_exif
    }

    /// 记录 undo
    pub fn push_undo(&mut self, tag: ExifTag, old_value: ExifValue, new_value: ExifValue) {
        self.undo_stack.push((tag, old_value, new_value));
        // 限制 undo 栈大小
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
    }

    /// 撤销
    pub fn undo(&mut self) -> bool {
        if let Some((tag, old_value, _)) = self.undo_stack.pop() {
            self.exif_entries.insert(tag, old_value);
            true
        } else {
            false
        }
    }

    /// 设置状态消息
    pub fn set_status(&mut self, message: impl Into<String>, level: StatusLevel) {
        self.status_message = Some((message.into(), level));
    }

    /// 清除状态消息
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}

// ============================================================================
// 导出
// ============================================================================

// pub use exif_entry::*;
// pub use image_state::*;

mod exif_entry;
mod image_state;
