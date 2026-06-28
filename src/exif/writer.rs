use crate::model::{ExifTag, ExifValue};
use anyhow::{Context, Result};
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

        // 构建 EXIF 字节流
        let new_exif = Self::build_exif(entries);

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

    /// 从 entries 构建 EXIF APP1 数据（简化版：直接构建 JPEG APP1 段）
    fn build_exif(entries: &HashMap<ExifTag, ExifValue>) -> Vec<u8> {
        // 构建简化的 EXIF 数据
        // 格式: "Exif\0\0" + TIFF header + IFD entries
        let mut exif = Vec::new();

        // EXIF header
        exif.extend_from_slice(b"Exif\0\0");

        // TIFF header (little endian)
        exif.extend_from_slice(&[0x49, 0x49]); // "II" little endian
        exif.extend_from_slice(&0x002Au16.to_le_bytes()); // TIFF magic
        exif.extend_from_slice(&0x0008u32.to_le_bytes()); // offset to first IFD

        // IFD0
        let entry_count = entries.len() as u16;
        exif.extend_from_slice(&entry_count.to_le_bytes());

        // 估算数据区偏移量: 8 (header) + 2 (count) + 12 * n (entries) + 4 (next IFD)
        let data_offset = 8 + 2 + 12 * entries.len() + 4;
        let mut data_area = Vec::new();
        let mut data_cursor = data_offset as u32;

        for (tag, value) in entries {
            // 12 bytes per entry: tag(2) + type(2) + count(4) + value/offset(4)
            exif.extend_from_slice(&tag.id.to_le_bytes());

            let (type_id, data_bytes) = Self::value_to_bytes(value);
            exif.extend_from_slice(&type_id.to_le_bytes());

            let count = data_bytes.len() as u32;
            exif.extend_from_slice(&count.to_le_bytes());

            if data_bytes.len() <= 4 {
                // 内联存储
                let mut inline = data_bytes.clone();
                inline.resize(4, 0u8);
                exif.extend_from_slice(&inline);
            } else {
                // 偏移量指向数据区
                exif.extend_from_slice(&data_cursor.to_le_bytes());
                data_area.extend_from_slice(&data_bytes);
                // 对齐到偶数
                if data_area.len() % 2 != 0 {
                    data_area.push(0);
                }
                data_cursor = data_offset as u32 + data_area.len() as u32;
            }
        }

        // Next IFD offset (0 = no more IFDs)
        exif.extend_from_slice(&0x00000000u32.to_le_bytes());

        // 数据区
        exif.extend_from_slice(&data_area);

        exif
    }

    fn value_to_bytes(value: &ExifValue) -> (u16, Vec<u8>) {
        match value {
            ExifValue::Byte(v) => (1, v.clone()),
            ExifValue::Ascii(s) => {
                let mut bytes = s.as_bytes().to_vec();
                bytes.push(0); // null terminator
                (2, bytes)
            }
            ExifValue::Short(v) => (3, v.to_le_bytes().to_vec()),
            ExifValue::Long(v) => (4, v.to_le_bytes().to_vec()),
            ExifValue::Rational(n, d) => (5, [n.to_le_bytes(), d.to_le_bytes()].concat()),
            ExifValue::SRational(n, d) => (10, [n.to_le_bytes(), d.to_le_bytes()].concat()),
            ExifValue::Undefined(v) => (7, v.clone()),
            ExifValue::Slice(v) => (7, v.clone()),
        }
    }

    /// 将新 EXIF 数据合并到原始 JPEG 文件数据中
    fn merge_exif(original: &[u8], new_exif: &[u8]) -> Result<Vec<u8>> {
        // 检查是否是 JPEG
        if original.len() < 2 || &original[0..2] != &[0xFF, 0xD8] {
            // 非 JPEG，直接返回原始数据
            return Ok(original.to_vec());
        }

        // 扫描并移除现有 APP1 EXIF 段，保留其他段
        let mut result = Vec::with_capacity(original.len() + new_exif.len());
        result.extend_from_slice(&original[0..2]); // SOI

        let mut i = 2;
        while i < original.len().saturating_sub(1) {
            if original[i] != 0xFF {
                i += 1;
                continue;
            }

            let marker = original[i + 1];

            if marker == 0x00 || marker == 0xFF {
                i += 1;
                continue;
            }

            if marker == 0xD8 {
                // SOI (shouldn't appear here)
                i += 2;
                continue;
            }

            if marker == 0xDA {
                // SOS — image data starts, copy rest
                result.extend_from_slice(&original[i..]);
                break;
            }

            if marker == 0xD9 {
                // EOI
                result.extend_from_slice(&original[i..]);
                break;
            }

            // 需要读取段长度
            if i + 3 >= original.len() {
                result.extend_from_slice(&original[i..]);
                break;
            }

            let seg_len = ((original[i + 2] as u16) << 8) | (original[i + 3] as u16);
            let seg_total = 2 + seg_len as usize;

            if marker == 0xE1 {
                // APP1 — 检查是否是 EXIF
                let seg_start = i + 4;
                if seg_start + 6 <= original.len()
                    && &original[seg_start..seg_start + 6] == b"Exif\0\0"
                {
                    // 跳过现有 EXIF
                    i += seg_total;
                    continue;
                }
            }

            // 保留其他段
            if i + seg_total <= original.len() {
                result.extend_from_slice(&original[i..i + seg_total]);
                i += seg_total;
            } else {
                result.extend_from_slice(&original[i..]);
                break;
            }
        }

        // 在 SOI 后插入新 APP1 EXIF 段
        let mut final_data = Vec::with_capacity(result.len() + new_exif.len() + 4);
        final_data.extend_from_slice(&result[0..2]); // SOI

        // APP1 marker + length
        let exif_len = (new_exif.len() + 2) as u16;
        final_data.push(0xFF);
        final_data.push(0xE1);
        final_data.extend_from_slice(&exif_len.to_be_bytes());
        final_data.extend_from_slice(new_exif);

        // 剩余数据（跳过 SOI）
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
            if data[i] != 0xFF {
                i += 1;
                continue;
            }

            let marker = data[i + 1];

            if marker == 0x00 || marker == 0xFF {
                i += 1;
                continue;
            }

            if marker == 0xDA {
                // SOS — image data starts
                result.extend_from_slice(&data[i..]);
                break;
            }

            if marker == 0xD9 {
                result.extend_from_slice(&data[i..]);
                break;
            }

            if i + 3 >= data.len() {
                break;
            }

            let seg_len = ((data[i + 2] as u16) << 8) | (data[i + 3] as u16);
            let seg_total = 2 + seg_len as usize;

            if marker == 0xE1 {
                // 跳过 APP1 (EXIF)
                let seg_start = i + 4;
                if seg_start + 6 <= data.len()
                    && &data[seg_start..seg_start + 6] == b"Exif\0\0"
                {
                    i += seg_total;
                    continue;
                }
            }

            // 保留其他段
            if i + seg_total <= data.len() {
                result.extend_from_slice(&data[i..i + seg_total]);
                i += seg_total;
            } else {
                result.extend_from_slice(&data[i..]);
                break;
            }
        }

        let tmp_path = Self::tmp_path(path);
        fs::write(&tmp_path, result)?;
        fs::rename(&tmp_path, path)?;

        Ok(())
    }
}
