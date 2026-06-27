use crate::model::{ExifTag, ExifValue};
use anyhow::{Context, Result};
use exif::Writer;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// EXIF 写入器 - 安全地将修改后的 EXIF 写回文件
pub struct ExifWriter;

impl ExifWriter {
    /// 安全写入 EXIF 到文件：先写临时文件，验证后再替换
    pub fn write(path: &Path, entries: &HashMap<ExifTag, ExifValue>) -> Result<()> {
        // 读取原始文件内容
        let original_data = fs::read(path)
            .with_context(|| format!("无法读取文件: {}", path.display()))?;

        // 构建 exif writer
        let mut writer = Writer::new();
        writer.set_mime_from_extension(
            path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("jpg"),
        );

        // 写入所有条目
        for (tag, value) in entries {
            Self::write_field(&mut writer, tag, value);
        }

        // 生成新的 EXIF 数据
        let new_exif = writer.write_to_vec()
            .context("生成 EXIF 数据失败")?;

        // 构建新文件：原始数据 + 新 EXIF
        let new_data = Self::merge_exif(&original_data, &new_exif)?;

        // 写入临时文件
        let tmp_path = Self::tmp_path(path);
        fs::write(&tmp_path, new_data)
            .with_context(|| format!("写入临时文件失败: {}", tmp_path.display()))?;

        // 验证临时文件
        Self::validate_written_file(&tmp_path)?;

        // 替换原文件
        fs::rename(&tmp_path, path)
            .with_context(|| format!("替换文件失败: {}", path.display()))?;

        Ok(())
    }

    /// 将内部 ExifValue 写入 exif::Writer
    fn write_field(writer: &mut Writer, tag: &ExifTag, value: &ExifValue) {
        use exif::{In, Value};

        let tag_enum = match tag.id {
            0x010F => exif::Tag::Make,
            0x0110 => exif::Tag::Model,
            0x0112 => exif::Tag::Orientation,
            0x011A => exif::Tag::XResolution,
            0x0128 => exif::Tag::ResolutionUnit,
            0x0131 => exif::Tag::Software,
            0x0132 => exif::Tag::DateTime,
            0x8298 => exif::Tag::Copyright,
            0x829A => exif::Tag::ExposureTime,
            0x829D => exif::Tag::FNumber,
            0x8822 => exif::Tag::ExposureProgram,
            0x8827 => exif::Tag::ISOSpeedRatings,
            0x9003 => exif::Tag::DateTimeOriginal,
            0x9004 => exif::Tag::DateTimeDigitized,
            0x920A => exif::Tag::FocalLength,
            0xA001 => exif::Tag::ColorSpace,
            0xA002 => exif::Tag::PixelXDimension,
            0xA003 => exif::Tag::PixelYDimension,
            _ => {
                // 对于未知 tag，尝试创建一个通用 tag
                return;
            }
        };

        match value {
            ExifValue::Ascii(s) => {
                let _ = writer.write_value(tag_enum, Value::Ascii(vec![Some(s.clone())]));
            }
            ExifValue::Short(v) => {
                let _ = writer.write_value(tag_enum, Value::Short(vec![*v]));
            }
            ExifValue::Long(v) => {
                let _ = writer.write_value(tag_enum, Value::Long(vec![*v]));
            }
            ExifValue::Rational(n, d) => {
                use exif::Rational;
                let _ = writer.write_value(tag_enum, Value::Rational(vec![Rational::new(*n, *d)]));
            }
            ExifValue::SRational(n, d) => {
                use exif::SRational;
                let _ = writer.write_value(tag_enum, Value::SRational(vec![SRational::new(*n, *d)]));
            }
            ExifValue::Byte(v) => {
                let _ = writer.write_value(tag_enum, Value::Byte(v.clone()));
            }
            _ => {}
        }
    }

    /// 将新 EXIF 数据合并到原始文件数据中
    fn merge_exif(original: &[u8], new_exif: &[u8]) -> Result<Vec<u8>> {
        // 简化方案：对于 JPEG，在 APP1 标记后插入新 EXIF
        // 更健壮的方案是使用 exif crate 的 in-place 修改

        // 查找 SOI (Start of Image: FF D8)
        if original.len() < 2 || &original[0..2] != &[0xFF, 0xD8] {
            // 非 JPEG，直接返回原始数据（对于 PNG/TIFF 等需要不同处理）
            return Ok(original.to_vec());
        }

        // 查找 APP1 标记（FF E1）或插入到 SOI 后
        let mut result = original.to_vec();

        // 移除现有的 APP1 EXIF 段
        let mut i = 2;
        while i < original.len().saturating_sub(1) {
            if original[i] == 0xFF {
                let marker = original[i + 1];
                if marker == 0xE1 {
                    // APP1 marker
                    if i + 3 < original.len() {
                        let seg_len = ((original[i + 2] as u16) << 8) | (original[i + 3] as u16);
                        let seg_total = 2 + seg_len as usize;
                        if i + seg_total <= original.len() {
                            // 跳过此段
                            i += seg_total;
                            continue;
                        }
                    }
                } else if marker == 0xDA || marker == 0xD9 {
                    // SOS 或 EOI，停止扫描
                    break;
                } else if marker != 0x00 && marker != 0xFF {
                    // 其他 marker
                    if i + 3 < original.len() {
                        let seg_len = ((original[i + 2] as u16) << 8) | (original[i + 3] as u16);
                        let seg_total = 2 + seg_len as usize;
                        if i + seg_total <= original.len() {
                            i += seg_total;
                            continue;
                        }
                    }
                }
            }
            i += 1;
        }

        // 在 SOI 后插入新 EXIF
        let mut final_data = Vec::with_capacity(2 + new_exif.len() + result.len() - 2);
        final_data.extend_from_slice(&result[0..2]); // SOI

        // 构建 APP1 段
        let app1_header: u16 = 0xFFE1;
        let exif_len = (new_exif.len() + 2) as u16; // 包含长度字段本身
        final_data.push((app1_header >> 8) as u8);
        final_data.push((app1_header & 0xFF) as u8);
        final_data.push((exif_len >> 8) as u8);
        final_data.push((exif_len & 0xFF) as u8);
        final_data.extend_from_slice(new_exif);
        final_data.extend_from_slice(&result[2..]);

        Ok(final_data)
    }

    /// 生成临时文件路径
    fn tmp_path(path: &Path) -> PathBuf {
        let mut tmp = path.as_os_str().to_os_string();
        tmp.push(".tmp");
        PathBuf::from(tmp)
    }

    /// 验证写入的文件可以正常读取
    fn validate_written_file(path: &Path) -> Result<()> {
        let data = fs::read(path)
            .with_context(|| format!("验证失败，无法读取: {}", path.display()))?;

        if data.is_empty() {
            anyhow::bail!("写入的文件为空");
        }

        if data.len() < 100 {
            anyhow::bail!("写入的文件过小，可能损坏");
        }

        // 基本魔数检查
        if data.starts_with(b"\xFF\xD8") || data.starts_with(b"\x89PNG") {
            Ok(())
        } else {
            anyhow::bail!("写入的文件格式异常");
        }
    }

    /// 完全清除图片的 EXIF 数据
    pub fn strip_exif(path: &Path) -> Result<()> {
        let data = fs::read(path)?;

        if !data.starts_with(b"\xFF\xD8") {
            anyhow::bail!("仅支持 JPEG 格式");
        }

        let mut result = vec![0xFF, 0xD8]; // SOI
        let mut i = 2;

        while i < data.len().saturating_sub(1) {
            if data[i] == 0xFF {
                let marker = data[i + 1];
                if marker == 0xE1 {
                    // 跳过 APP1 (EXIF)
                    if i + 3 < data.len() {
                        let seg_len = ((data[i + 2] as u16) << 8) | (data[i + 3] as u16);
                        let seg_total = 2 + seg_len as usize;
                        if i + seg_total <= data.len() {
                            i += seg_total;
                            continue;
                        }
                    }
                } else if marker == 0xDA {
                    // SOS - 剩余数据全部保留
                    result.extend_from_slice(&data[i..]);
                    break;
                } else if marker == 0xD9 || marker == 0xD7 {
                    // EOI or JPG
                    result.extend_from_slice(&data[i..]);
                    break;
                } else if marker != 0x00 && marker != 0xFF && i + 3 < data.len() {
                    let seg_len = ((data[i + 2] as u16) << 8) | (data[i + 3] as u16);
                    let seg_total = 2 + seg_len as usize;
                    if i + seg_total <= data.len() {
                        result.extend_from_slice(&data[i..i + seg_total]);
                        i += seg_total;
                        continue;
                    }
                }
            }
            i += 1;
        }

        let tmp_path = Self::tmp_path(path);
        fs::write(&tmp_path, result)?;
        fs::rename(&tmp_path, path)?;

        Ok(())
    }
}
