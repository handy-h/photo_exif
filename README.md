# Photo EXIF Tool (v0.1.1)

一款轻量级桌面应用，用于查看、编辑和管理相片的 EXIF 信息。支持单图精细化编辑和批量操作，特别注重隐私保护和格式校验。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-2021+-orange.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)

## 功能特性

### 核心功能
- 📷 **图片预览** — 支持 JPEG、PNG、WebP、TIFF、BMP 格式
- 📝 **EXIF 查看** — 以表格形式展示所有 EXIF 字段，支持分组折叠
- ✏️ **EXIF 编辑** — 双击字段值即可编辑，智能格式化（如曝光时间、光圈值）
- 💾 **安全保存** — 原子写入（先写临时文件，验证后替换），避免文件损坏
- 🔄 **重置功能** — 一键恢复原始 EXIF 信息
- 🔍 **格式校验** — 检测文件 magic bytes 与扩展名是否匹配

### 批量操作
- 📂 **批量清空 EXIF** — 一键清除所有/仅清除 GPS/隐私字段
- 📋 **EXIF 复制粘贴** — 从一张图复制 EXIF 粘贴到另一张
- 🏷️ **按 EXIF 重命名** — 根据拍摄时间、ISO、光圈等重命名文件
- 📤 **批量导出** — 导出整个文件夹的 EXIF 为 JSON/CSV

### 高级功能
- 🗺️ **GPX GPS 写入** — 加载 GPX 轨迹，根据拍摄时间匹配写入 GPS 坐标
- 🔀 **EXIF 对比模式** — 并排显示两张照片的 EXIF 差异
- 🔧 **EXIF 校验修复** — 检测损坏的 EXIF 并尝试修复
- ⚠️ **隐私风险提示** — 检测到 GPS/个人信息时红色高亮警告
- 🖱️ **拖放支持** — 拖拽文件夹、图片或 GPX 文件到窗口直接打开

### 用户体验
- ⌨️ **键盘快捷键** — ←→ 切换、Ctrl+S 保存、+/- 缩放、F 全屏、1 实际像素
- 📜 **最近打开记忆** — 自动恢复上次打开的文件夹和位置
- 🔎 **字段搜索** — 快速定位 EXIF 标签
- 🎨 **中文界面** — 完整的中文标签名和界面

## 截图


## 安装

### 从源码编译

需要 Rust 1.70+ 环境。

```bash
# 克隆仓库
git clone git@github.com:handy-h/photo_exif.git
cd photo_exif

# 编译
cargo build --release

# 运行
./target/release/photo_exit
```

### 预编译二进制

从 [Releases](https://github.com/handy-h/photo_exif/releases) 页面下载对应平台的可执行文件。

## 使用指南

### 基本操作
- **打开文件夹**：Ctrl+O 或点击工具栏"打开文件夹"
- **打开单个文件**：Ctrl+N 或点击工具栏"打开文件"
- **切换图片**：← → 方向键或点击左右按钮
- **保存修改**：Ctrl+S 或点击"保存"按钮
- **重置修改**：点击"重置"按钮恢复原始 EXIF

### 编辑 EXIF
1. 在右侧面板找到要编辑的字段
2. 双击 Value 列的值进入编辑模式
3. 输入新值后按 Enter 确认，或按 Esc 取消
4. 点击"保存"将修改写回文件

### 批量操作
1. 点击工具栏"批量操作"按钮
2. 选择操作类型（清空 EXIF、重命名等）
3. 根据提示完成操作

### GPX GPS 写入
1. 点击工具栏"📍 GPX写入"按钮
2. 选择 GPX 轨迹文件
3. 预览匹配结果
4. 点击"写入当前图片"或"批量写入所有"

### 隐私保护
- 检测到 GPS 信息时，GPS 分组会红色高亮显示
- 点击"批量操作" → "一键脱敏"可快速清除隐私信息

## 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| Ctrl+O | 打开文件夹 |
| Ctrl+N | 打开单个文件 |
| Ctrl+S | 保存 EXIF 修改 |
| Ctrl+Z | 撤销上次修改 |
| ← / → | 切换上一张/下一张 |
| +/- | 缩放预览图 |
| 1 | 切换 1:1 实际像素视图 |
| F | 切换全屏模式 |
| Del | 删除选中的 EXIF 字段 |
| Ctrl+Shift+C | 复制当前图片的 EXIF |
| Ctrl+Shift+V | 粘贴 EXIF 到当前图片 |
| Ctrl+Shift+G | 打开 GPX 写入窗口 |
| Ctrl+Shift+D | 打开 EXIF 对比窗口 |
| Ctrl+Shift+R | 打开 EXIF 修复窗口 |

## 技术栈

- **语言**：Rust 2021+
- **GUI 框架**：eframe 0.27 (egui)
- **图片处理**：image 0.25
- **EXIF 读写**：exif 0.6
- **文件对话框**：rfd 0.14

## 项目结构

```
src/
├── main.rs              # 程序入口
├── lib.rs               # 库入口
├── app.rs               # 主应用逻辑
├── model/               # 数据模型
│   └── mod.rs           # AppState, ExifTag, ExifValue 等
├── ui/                  # UI 组件
│   ├── preview.rs       # 图片预览
│   ├── exif_panel.rs    # EXIF 面板
│   ├── toolbar.rs       # 工具栏
│   ├── left_panel.rs    # 左侧缩略图列表
│   ├── thumbnail_bar.rs # 底部缩略图画廊
│   ├── shortcuts.rs     # 键盘快捷键
│   ├── compare.rs       # EXIF 对比模式
│   ├── gpx_window.rs    # GPX 写入窗口
│   └── repair_window.rs # EXIF 修复窗口
├── exif/                # EXIF 处理
│   ├── reader.rs        # 读取 EXIF
│   ├── writer.rs        # 写入 EXIF
│   ├── formatter.rs     # 值格式化
│   ├── validator.rs     # 格式校验
│   ├── gpx.rs           # GPX 解析
│   └── repair.rs        # EXIF 修复
├── io/                  # 文件操作
│   ├── image_loader.rs  # 图片加载
│   └── file_ops.rs      # 文件操作
└── config/              # 配置
    └── settings.rs      # 设置持久化
```

## 开发计划

- [x] v0.1 MVP：打开、预览、EXIF 列表、编辑、保存
- [x] v0.2：分组、搜索、快捷键、隐私提示
- [x] v0.3：缩略图、批量操作、拖放、记忆
- [x] v0.4：GPX 写入、对比模式、EXIF 修复
- [ ] v1.0：直方图、RAW 支持、更多格式

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

本项目采用 [MIT 许可证](LICENSE) 开源。

```
MIT License

Copyright (c) 2026 Photo EXIF Tool Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## 致谢

- [egui](https://github.com/emilk/egui) — 即时模式 GUI 库
- [exif-rs](https://github.com/kamadak/exif-rs) — Rust EXIF 库
- [image](https://github.com/image-rs/image) — Rust 图像处理库
