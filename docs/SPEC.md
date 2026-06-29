# Photo EXIF Tool 规格说明书

> **版本**: v0.1.1  
> **状态**: Draft  
> **技术栈**: Rust + egui/eframe  

---

## 1. 项目概述

一款轻量级桌面应用，用于查看、编辑和管理相片的 EXIF 信息。支持单图精细化编辑和批量操作，特别注重隐私保护和格式校验。

**核心定位**: 替代现有 EXIF 查看工具的日常使用场景，提供更快的编辑体验和更安全的写入流程。

---

## 2. 目标用户

- 摄影爱好者：需要批量调整拍摄参数或统一版权信息
- 隐私意识强的用户：发布照片前清除 GPS/个人信息
- 图片整理者：按拍摄时间重命名、批量整理照片
- 开发者/设计师：需要精确查看 EXIF 原始数据

---

## 3. 功能规格

### 3.1 核心功能（v0.1.1 — MVP）

| 功能 | 描述 | 优先级 |
|------|------|--------|
| 打开图片 | 支持 JPEG、PNG、WebP、TIFF、BMP | P0 |
| 图片预览 | 左侧等比缩放预览，支持滚轮缩放 | P0 |
| EXIF 列表 | 右侧以表格形式展示所有 EXIF 字段 | P0 |
| 字段编辑 | Tag 列只读，Value 列为可编辑文本框 | P0 |
| 保存 | 将修改后的 EXIF 写回文件（安全写入） | P0 |
| 重置 | 放弃当前修改，恢复文件原始 EXIF | P0 |
| 上下切换 | 同一文件夹内前后切换照片 | P0 |
| 格式校验 | 检测 magic bytes 与扩展名是否匹配，不匹配时警告 | P0 |

### 3.2 版本 1 功能（v0.2）

| 功能 | 描述 | 优先级 |
|------|------|--------|
| 字段分组 | 按「相机信息 / 曝光参数 / GPS / 其他」分组折叠 | P1 |
| 智能格式化 | 原始值 `300/100` 显示为 `3.0`，编辑后转回原始类型 | P1 |
| 快捷编辑面板 | 单独提供日期时间、ISO、光圈、快门、焦距、GPS 快速编辑区 | P1 |
| 字段搜索 | 输入 tag 名称快速过滤定位 | P1 |
| 键盘快捷键 | ←→ 切换、Ctrl+S 保存、+/- 缩放、F 全屏、Del 删除 | P1 |
| 删除字段 | 删除指定 EXIF tag | P1 |
| 隐私风险提示 | 检测到 GPS/个人信息时红色高亮警告 | P1 |
| 写入确认 | Save 前列出即将修改/删除的字段，二次确认 | P1 |

### 3.3 版本 2 功能（v0.3 — 批量能力）

| 功能 | 描述 | 优先级 |
|------|------|--------|
| 底部缩略图画廊 | ScrollArea + 小图 grid，直观浏览文件夹 | P2 |
| 批量清空 EXIF | 选中多张 → 一键清除所有/仅清除 GPS/隐私字段 | P2 |
| EXIF 复制粘贴 | 从一张图复制 → 粘贴到另一张 | P2 |
| 批量按 EXIF 重命名 | `20240101_120000_ISO100_f2.8.jpg` 格式 | P2 |
| 拖放打开 | 拖拽文件夹或图片到窗口打开 | P2 |
| 最近打开记忆 | 启动时恢复上次打开的文件夹和位置 | P2 |
| 1:1 像素视图 | 按 `1` 键切换到实际像素大小 | P2 |
| 全屏模式 | 隐藏所有面板，仅显示大图 | P2 |

### 3.4 版本 3 功能（v0.4+）

| 功能 | 描述 | 优先级 |
|------|------|--------|
| 批量导出 EXIF | 整个文件夹导出为 JSON / CSV | P3 |
| 一键脱敏 | 一键删除 GPS、相机序列号、镜头序列号、缩略图 EXIF | P3 |
| GPX GPS 写入 | 加载 GPX 轨迹，根据拍摄时间匹配经纬度 | P3 |
| EXIF 对比模式 | 并排显示两张照片的 EXIF 差异 | P3 |
| 直方图叠加 | 预览图上叠加 RGB 直方图 | P3 |
| RAW 缩略图支持 | 用嵌入的 JPEG 缩略图预览 CR2/NEF/ARW | P3 |
| EXIF 校验和修复 | 检测损坏的 EXIF 并尝试修复 | P3 |

---

## 4. 技术架构

### 4.1 技术栈

```
语言:     Rust 2021+
GUI:      eframe 0.27 (egui)
图片处理: image 0.25
EXIF:     exif 0.5 (kornelski/exif) — 读写
          kamadak_exif 0.7  — 只读解析（如需更完整的 tag 支持）
其他:     rfd (文件对话框), serde (配置持久化)
```

### 4.2 模块划分

```
src/
├── main.rs              # 入口
├── app.rs               # App 状态、事件循环
├── ui/
│   ├── mod.rs
│   ├── preview.rs       # 左侧图片预览 + 缩放
│   ├── exif_panel.rs    # 右侧 EXIF 表格 + 编辑
│   ├── toolbar.rs       # 顶部/底部按钮栏
│   ├── thumbnail_bar.rs # 底部缩略图画廊
│   └── shortcuts.rs     # 键盘快捷键处理
├── exif/
│   ├── mod.rs
│   ├── reader.rs        # 读取 EXIF，返回结构化数据
│   ├── writer.rs        # 写入 EXIF，安全写入流程
│   ├── formatter.rs     # 值格式化（Rational → 显示字符串）
│   └── validator.rs     # 格式/扩展名校验
├── model/
│   ├── mod.rs
│   ├── image_state.rs   # 当前图片、缩放比例、原始 EXIF 快照
│   └── exif_entry.rs    # ExifEntry { tag, value, original_value }
├── io/
│   ├── mod.rs
│   ├── image_loader.rs  # 加载图片 + 检测格式
│   ├── file_ops.rs      # 保存、重置、重命名、批量操作
│   └── batch.rs         # 批量处理逻辑
└── config/
    ├── mod.rs
    └── settings.rs      # 最近打开记录、忽略的扩展名警告
```

### 4.3 数据流

```
用户打开文件
  │
  ▼
ImageReader::open(path)
  │
  ├─▶ with_guessed_format() ──▶ 检测实际格式
  │     │
  │     ├─ 格式不匹配扩展名 ──▶ 显示警告 + 修正按钮
  │     └─ 匹配 ─────────────────────▶ 继续
  │
  ▼
加载为 DynamicImage ──▶ 缩放到预览尺寸 ──▶ egui::Image
  │
  ▼
读取 EXIF ──▶ HashMap<Tag, Value> ──▶ 结构化显示
  │
  ▼
用户编辑 Value ──▶ 存入临时 HashMap（不写回文件）
  │
  ▼
用户点击 Save
  │
  ├─▶ 二次确认对话框（列出变更）
  │     │
  │     ▼
  ├─▶ 写临时文件: path.tmp
  │     │
  │     ▼
  ├─▶ 验证临时文件可读
  │     │
  │     ▼
  └─▶ 替换原文件: remove(path) + rename(tmp → path)
```

---

## 5. UI/UX 设计

### 5.1 主界面布局

```
┌─────────────────────────────────────────────────────────┐
│  [打开文件夹] [打开文件]              photo_exit v0.1.1.0  │
├───────────────────────┬─────────────────────────────────┤
│                       │  ⚠️ 实际格式 JPEG，扩展名 .png  │
│                       │  [修正扩展名]                    │
│                       │                                 │
│   图片预览区域         │  🔍 搜索字段...                  │
│   (自适应填充)         │                                 │
│                       │  相机信息                        │
│                       │  ├── Make      │ Canon        ✎  │
│   滚轮：缩放           │  ├── Model     │ EOS R5       ✎  │
│   拖拽：移动           │  └── ...       │ ...          ✎  │
│                       │                                 │
│                       │  曝光参数                        │
│                       │  ├── ISO       │ 100          ✎  │
│                       │  ├── FNumber   │ 28/10        ✎  │
│                       │  └── ...       │ ...          ✎  │
│                       │                                 │
│  [◀ Prev] [Next ▶]   │  GPS                            │
│  [Reset]   [Save]     │  ├── GPSLatitude │ ...        ✎  │
│                       │  └── ...         │ ...        ✎  │
│                       │                                 │
│                       │  [删除选中字段]                  │
├───────────────────────┴─────────────────────────────────┤
│  [缩略图1] [缩略图2] [缩略图3] ...                       │
└─────────────────────────────────────────────────────────┘
```

### 5.2 交互规范

| 操作 | 行为 |
|------|------|
| 双击字段 Value | 进入编辑模式 |
| 编辑时按 Enter | 确认修改 |
| 编辑时按 Esc | 取消编辑 |
| 点击字段左侧 | 选中（支持批量删除） |
| Ctrl+点击 | 多选 |
| Delete | 删除选中字段 |
| Ctrl+S | 保存（带确认对话框） |
| Ctrl+Z | 撤销上次修改（最近 N 步） |
| Ctrl+Shift+C | 复制选中字段的 EXIF |
| Ctrl+Shift+V | 粘贴 EXIF 到当前图片 |
| 1 | 切换 1:1 像素视图 |
| F | 全屏切换 |
| ← → | 切换上一张/下一张 |
| +/- | 缩放预览图 |
| Ctrl+O | 打开文件夹 |
| Ctrl+N | 打开单个文件 |

### 5.3 视觉规范

- 警告/错误状态使用颜色区分：
  - 红色：隐私风险（GPS/个人信息）、写入失败
  - 黄色：格式不匹配、字段值异常
  - 绿色：保存成功、操作完成
- 编辑中的字段：蓝色边框高亮
- 分组折叠：默认展开「相机信息」和「曝光参数」，折叠「其他」

---

## 6. 数据模型

### 6.1 核心结构

```rust
struct AppState {
    // 文件列表
    folder_path: Option<PathBuf>,
    file_paths: Vec<PathBuf>,
    current_index: usize,
    
    // 当前图片
    current_image: Option<DynamicImage>,
    original_format: Option<ImageFormat>,
    
    // EXIF 数据
    exif_entries: HashMap<ExifTag, ExifValue>,
    original_exif: HashMap<ExifTag, ExifValue>,  // 用于 Reset
    undo_stack: Vec<(ExifTag, ExifValue, ExifValue)>, // (tag, old, new)
    
    // UI 状态
    search_query: String,
    expanded_groups: HashSet<ExifGroup>,
    selected_tags: HashSet<ExifTag>,
    zoom: f32,
    is_fullscreen: bool,
    
    // 警告
    extension_warning: Option<String>, // "实际 JPEG，扩展名 .png"
    
    // 批量操作
    clipboard_exif: Option<HashMap<ExifTag, ExifValue>>,
}

enum ExifGroup {
    CameraInfo,
    Exposure,
    GPS,
    Lens,
    Thumbnail,
    Other,
}

enum ExifValue {
    Byte(Vec<u8>),
    Ascii(String),
    Short(u16),
    Long(u32),
    Rational(u32, u32),      // 显示为 "28/10" 或 "2.8"
    SRational(i32, i32),
    Undefined(Vec<u8>),
    Slice(Vec<u8>),
}
```

### 6.2 EXIF 映射

常用 tag 的显示名映射（内置字典）：

```rust
const TAG_NAMES: &[(u16, &str)] = &[
    (0x010F, "制造商"),
    (0x0110, "型号"),
    (0x0112, "方向"),
    (0x011A, "水平分辨率"),
    (0x0131, "软件"),
    (0x0132, "修改时间"),
    (0x829A, "曝光时间"),
    (0x829D, "光圈值"),
    (0x8827, "ISO"),
    (0x9003, "拍摄时间"),
    (0x9004, "数字化时间"),
    (0x920A, "焦距"),
    (0xA001, "颜色空间"),
    (0xA002, "像素宽"),
    (0xA003, "像素高"),
    // GPS tags...
];
```

---

## 7. 外部依赖

### 7.1 生产依赖

| 依赖 | 版本 | 用途 | 备注 |
|------|------|------|------|
| eframe | 0.27 | GUI 框架 | 含 egui |
| image | 0.25 | 图片加载/解码/编码 | 支持格式检测 |
| exif | 0.5 | EXIF 读写 | 支持写入 JPEG/PNG/TIFF |
| rfd | 0.14 | 原生文件对话框 | 打开文件/文件夹 |
| chrono | 0.4 | 日期时间处理 | EXIF 时间格式化 |
| anyhow | 1.0 | 错误处理 | 简化 error 传递 |
| serde + serde_json | 1.0 | 配置持久化 | 最近打开记录 |

### 7.2 开发依赖

| 依赖 | 用途 |
|------|------|
| clippy | 代码检查 |
| rustfmt | 代码格式化 |

---

## 8. 非功能需求

### 8.1 性能

- 打开 < 20MB JPEG：< 500ms（含 EXIF 解析）
- 切换照片：< 200ms
- 保存 EXIF：< 1s（单文件）
- 内存占用：< 100MB（正常使用）

### 8.2 安全性

- **原子写入**：Save 时先写 `.tmp`，验证后再替换，避免文件损坏
- **不修改原始数据**：Reset 前保留完整原始 EXIF 快照
- **无网络请求**：纯本地应用，不上传任何数据
- **路径遍历防护**：批量操作时验证文件路径合法性

### 8.3 兼容性

- 支持操作系统：Windows 10+、macOS 12+、Ubuntu 20.04+
- 输出格式：单二进制文件，无外部运行时依赖
- 支持格式：JPEG、PNG、WebP、TIFF、BMP（v0.1.1）
- RAW 格式：v0.4+ 仅读取嵌入缩略图，不修改原始 RAW

### 8.4 可用性

- 启动时间：< 1s
- 首次打开提示快捷键说明
- 错误信息清晰（非内部 panic 信息）
- 支持中文界面和 tag 名翻译

---

## 9. 里程碑计划

| 版本 | 时间 | 目标 |
|------|------|------|
| v0.1.1 | 2 周 | MVP：打开、预览、EXIF 列表、编辑、保存、格式校验 |
| v0.2 | 3 周 | 分组、搜索、快捷键、隐私提示、写入确认 |
| v0.3 | 3 周 | 底部缩略图、批量清空、复制粘贴、拖放、记忆 |
| v0.4 | 4 周 | 导出、一键脱敏、1:1 视图、全屏 |
| v1.0 | - | GPX、对比模式、直方图、RAW 缩略图 |

---

## 10. 已知限制

1. **RAW 格式写入极难**：CR2/NEF/ARW 写入 EXIF 需要专用库或调用 `exiftool`，v1.0 前不承诺支持写入
2. **EXIF 写入会降低质量**：某些库写入 EXIF 会重新编码 JPEG，导致质量损失（通过 `exif` crate 的 sidecar 方式可缓解）
3. **大文件夹卡顿**：单文件夹 > 1000 张图片时，缩略图加载需要虚拟滚动优化
4. **GPS 坐标系转换**：EXIF 存储的是度分秒（DMS），需要转换为十进制显示，不同格式（WGS84 vs GCJ02）暂不处理
5. **只读字段**：部分 tag（如像素尺寸、压缩率）是只读的，编辑后不生效，需要提示用户

---

## 11. 格式/扩展名校验（特殊需求）

### 11.1 检测机制

```rust
fn validate_extension(path: &Path) -> Result<ValidationResult, Error> {
    let format = ImageReader::open(path)?
        .with_guessed_format()?
        .format();
    
    let expected_ext = format
        .extensions_str()
        .into_iter()
        .next()
        .unwrap_or("bin");
    
    let actual_ext = path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    
    if actual_ext == expected_ext {
        Ok(ValidationResult::Match)
    } else {
        Ok(ValidationResult::Mismatch {
            actual_format: format,
            expected_ext: expected_ext.to_string(),
            actual_ext,
        })
    }
}
```

### 11.2 展示逻辑

| 状态 | UI 表现 | 操作 |
|------|---------|------|
| 匹配 | 无提示 | 无 |
| 不匹配 | 黄色警告条 + 图标 | [修正扩展名] [忽略本次] |
| 检测失败 | 灰色提示 | 继续加载（可能非图片） |

### 11.3 修正逻辑

```
修正扩展名按钮点击
  │
  ▼
原文件名: photo.jpg.png
实际格式: JPEG
目标扩展名: .jpg
新文件名: photo.jpg
  │
  ▼
二次确认对话框
  │
  ├─ 确认 ──▶ rename("photo.jpg.png", "photo.jpg")
  │               └─ 更新 app_state.folder_paths
  │               └─ 重新加载当前文件
  │
  └─ 取消 ──▶ 无操作
```

### 11.4 边界处理

- **文件被占用**：rename 失败时提示"文件可能被其他程序占用"
- **重名冲突**：`photo.jpg` 已存在时提示覆盖或自动重命名为 `photo (1).jpg`
- **忽略记忆**：用户选择"忽略"后，记录到 `config/ignored_warnings.json`，下次打开同类文件不再提示

---

## 12. 项目结构（生成后）

```
photo_exit/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── ui/
│   ├── exif/
│   ├── model/
│   ├── io/
│   └── config/
├── docs/
│   └── SPEC.md          ← 本文档
└── assets/
    └── icons/           # 应用图标（可选）
```

---

## 13. 下一步行动

1. 初始化 Cargo 项目，配置 `Cargo.toml`
2. 实现 `ImageLoader` 和 `ExtensionValidator` 模块
3. 搭建基础 egui 布局（左右分栏）
4. 实现 EXIF 读取和表格展示
5. 实现保存/重置流程（重点：原子写入）
