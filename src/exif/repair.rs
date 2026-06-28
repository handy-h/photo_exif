use anyhow::{Context, Result};
use std::path::Path;

/// EXIF 完整性校验和修复
pub struct ExifRepairer;

impl ExifRepairer {
    /// 检测 EXIF 是否有效
    pub fn validate(path: &Path) -> Result<ExifHealth> {
        let data = std::fs::read(path)
            .with_context(|| format!("无法读取文件: {}", path.display()))?;

        let mut health = ExifHealth::default();

        // 基本格式检测
        health.format_valid = Self::check_format(&data);

        // JPEG 特定检测
        if data.starts_with(b"\xFF\xD8") {
            health.jfif_valid = Self::check_jfif(&data);
            health.exif_valid = Self::check_exif_integrity(&data);
            health.marker_valid = Self::check_markers(&data);
        }

        // 计算总体评分
        health.score = Self::calculate_score(&health);

        Ok(health)
    }

    fn check_format(data: &[u8]) -> bool {
        data.starts_with(b"\xFF\xD8")     // JPEG
            || data.starts_with(b"\x89PNG") // PNG
            || data.starts_with(b"II")      // TIFF little-endian
            || data.starts_with(b"MM")      // TIFF big-endian
    }

    fn check_jfif(data: &[u8]) -> bool {
        if !data.starts_with(b"\xFF\xD8") {
            return true; // 非 JPEG，跳过
        }

        // 检查 SOI
        if !data.starts_with(b"\xFF\xD8") {
            return false;
        }

        // 检查 APP0 (JFIF) 或 APP1 (EXIF) 标记
        let mut i = 2;
        while i < data.len().saturating_sub(1) {
            if data[i] != 0xFF {
                i += 1;
                continue;
            }
            let marker = data[i + 1];

            if marker == 0xD8 || marker == 0xFF || marker == 0x00 {
                i += 1;
                continue;
            }

            // SOS 或 EOI 之后不应该有其他标记
            if marker == 0xDA || marker == 0xD9 {
                return true;
            }

            if i + 3 >= data.len() {
                return false;
            }

            let seg_len = ((data[i + 2] as usize) << 8) | (data[i + 3] as usize);
            if seg_len < 2 || i + 2 + seg_len > data.len() {
                return false;
            }

            i += 2 + seg_len;
        }

        true
    }

    fn check_exif_integrity(data: &[u8]) -> bool {
        if !data.starts_with(b"\xFF\xD8") {
            return true;
        }

        // 查找 APP1 EXIF 段
        let mut i = 2;
        while i < data.len().saturating_sub(1) {
            if data[i] != 0xFF {
                i += 1;
                continue;
            }

            let marker = data[i + 1];
            if marker == 0xFF || marker == 0x00 || marker == 0xD8 {
                i += 1;
                continue;
            }

            if i + 3 >= data.len() {
                return false;
            }

            let seg_len = ((data[i + 2] as usize) << 8) | (data[i + 3] as usize);

            if marker == 0xE1 && i + 4 + 6 <= data.len() {
                // APP1 段
                let app1_start = i + 4;
                if &data[app1_start..app1_start + 6] == b"Exif\0\0" {
                    // 检查 TIFF header
                    let tiff_start = app1_start + 6;
                    if tiff_start + 8 > data.len() {
                        return false;
                    }

                    let endian = &data[tiff_start..tiff_start + 2];
                    if endian != b"II" && endian != b"MM" {
                        return false;
                    }

                    // TIFF magic: 0x002A
                    let magic = if endian == b"II" {
                        (data[tiff_start + 2] as u16) | ((data[tiff_start + 3] as u16) << 8)
                    } else {
                        ((data[tiff_start + 2] as u16) << 8) | (data[tiff_start + 3] as u16)
                    };

                    return magic == 0x002A;
                }
            }

            if marker == 0xDA {
                break;
            }

            i += 2 + seg_len;
        }

        true
    }

    fn check_markers(data: &[u8]) -> bool {
        if !data.starts_with(b"\xFF\xD8") {
            return true;
        }

        let mut found_sof = false;
        let mut i = 2;

        while i < data.len().saturating_sub(1) {
            if data[i] != 0xFF {
                i += 1;
                continue;
            }

            let marker = data[i + 1];

            // 跳过填充
            if marker == 0xFF || marker == 0x00 {
                i += 1;
                continue;
            }

            // SOS 之后的数据不需要解析
            if marker == 0xDA {
                return true;
            }

            // EOI 应该出现在数据末尾
            if marker == 0xD9 {
                return i + 2 == data.len() || i + 2 > data.len();
            }

            if i + 3 >= data.len() {
                return false;
            }

            let seg_len = ((data[i + 2] as usize) << 8) | (data[i + 3] as usize);

            // SOF 标记 (Start of Frame)
            if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xCC {
                found_sof = true;
            }

            // 无效段长度
            if seg_len < 2 {
                return false;
            }

            // 段长度超出文件
            if i + 2 + seg_len > data.len() {
                return false;
            }

            i += 2 + seg_len;
        }

        found_sof
    }

    fn calculate_score(health: &ExifHealth) -> f32 {
        let mut score: f32 = 100.0;

        if !health.format_valid {
            score -= 50.0;
        }
        if !health.jfif_valid {
            score -= 20.0;
        }
        if !health.exif_valid {
            score -= 20.0;
        }
        if !health.marker_valid {
            score -= 10.0;
        }

        score.max(0.0)
    }

    /// 尝试修复 EXIF
    pub fn repair(path: &Path) -> Result<RepairResult> {
        let data = std::fs::read(path)
            .with_context(|| format!("无法读取文件: {}", path.display()))?;

        if !data.starts_with(b"\xFF\xD8") {
            anyhow::bail!("仅支持 JPEG 格式的修复");
        }

        let mut repairs = Vec::new();
        let mut fixed = data.to_vec();

        // 修复 1: 确保有 SOI
        if !fixed.starts_with(b"\xFF\xD8") {
            repairs.push("添加缺失的 SOI 标记".to_string());
            fixed = [b"\xFF\xD8".to_vec(), fixed].concat();
        }

        // 修复 2: 移除尾部垃圾数据
        let trimmed = Self::trim_trailing_garbage(&fixed);
        if trimmed.len() < fixed.len() {
            repairs.push(format!(
                "移除尾部垃圾数据 ({} 字节)",
                fixed.len() - trimmed.len()
            ));
            fixed = trimmed;
        }

        // 修复 3: 清理格式错误的段
        let cleaned = Self::clean_malformed_segments(&fixed);
        if cleaned.len() != fixed.len() {
            repairs.push("清理格式错误的段".to_string());
            fixed = cleaned;
        }

        // 修复 4: 确保有 EOI
        if !fixed.ends_with(b"\xFF\xD9") {
            repairs.push("添加缺失的 EOI 标记".to_string());
            fixed.push(0xFF);
            fixed.push(0xD9);
        }

        // 写入临时文件并验证
        let tmp_path = {
            let mut p = path.as_os_str().to_os_string();
            p.push(".repair.tmp");
            std::path::PathBuf::from(p)
        };

        std::fs::write(&tmp_path, &fixed)?;

        // 验证修复后的文件
        if !Self::check_format(&fixed) || !Self::check_markers(&fixed) {
            std::fs::remove_file(&tmp_path)?;
            anyhow::bail!("修复后文件格式仍然异常");
        }

        // 原子替换
        std::fs::rename(&tmp_path, path)?;

        Ok(RepairResult {
            repairs,
            bytes_saved: data.len() as i64 - fixed.len() as i64,
        })
    }

    fn trim_trailing_garbage(data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }

        // 从后向前找到 EOI 或 SOS 结尾
        let mut end = data.len();

        // 尝试从末尾找到 EOI
        for i in (0..data.len().saturating_sub(1)).rev() {
            if data[i] == 0xFF && data.get(i + 1) == Some(&0xD9) {
                end = i + 2;
                break;
            }
            if data[i] == 0xFF && data.get(i + 1) == Some(&0xDA) {
                // SOS 后还有数据，需要找到 SOS 后真正的结束
                let sos_end = Self::find_sos_end(&data[i..]);
                if let Some(se) = sos_end {
                    end = i + se;
                    break;
                }
            }
        }

        data[..end].to_vec()
    }

    fn find_sos_end(data: &[u8]) -> Option<usize> {
        // SOS 后是压缩的图像数据，以 0xFF 0x00 或 0xFF 0xD9 结束
        let mut i = 0;
        while i < data.len().saturating_sub(1) {
            if data[i] == 0xFF {
                let next = data[i + 1];
                if next == 0x00 || next == 0xD9 {
                    return Some(i + 2);
                }
                if next != 0xFF && i + 3 < data.len() {
                    let seg_len = ((data[i + 2] as usize) << 8) | (data[i + 3] as usize);
                    i += 2 + seg_len;
                    continue;
                }
            }
            i += 1;
        }
        None
    }

    fn clean_malformed_segments(data: &[u8]) -> Vec<u8> {
        if !data.starts_with(b"\xFF\xD8") {
            return data.to_vec();
        }

        let mut result = vec![0xFF, 0xD8];
        let mut i = 2;

        while i < data.len().saturating_sub(1) {
            if data[i] != 0xFF {
                i += 1;
                continue;
            }

            let marker = data[i + 1];

            if marker == 0xFF || marker == 0x00 {
                i += 1;
                continue;
            }

            if marker == 0xD8 {
                i += 2;
                continue; // 重复的 SOI
            }

            if i + 3 >= data.len() {
                break;
            }

            let seg_len = ((data[i + 2] as usize) << 8) | (data[i + 3] as usize);

            // SOS 及其后续数据直接追加
            if marker == 0xDA {
                result.extend_from_slice(&data[i..]);
                break;
            }

            // EOI
            if marker == 0xD9 {
                result.extend_from_slice(&[0xFF, 0xD9]);
                break;
            }

            // 无效段长度
            if seg_len < 2 {
                i += 1;
                continue;
            }

            let seg_end = i + 2 + seg_len;
            if seg_end > data.len() {
                i += 1;
                continue;
            }

            // 有效段
            result.extend_from_slice(&data[i..seg_end]);
            i = seg_end;
        }

        result
    }
}

/// EXIF 健康状态
#[derive(Debug, Default)]
pub struct ExifHealth {
    pub score: f32,
    pub format_valid: bool,
    pub jfif_valid: bool,
    pub exif_valid: bool,
    pub marker_valid: bool,
}

impl ExifHealth {
    pub fn status_label(&self) -> &'static str {
        if self.score >= 90.0 {
            "✅ 良好"
        } else if self.score >= 70.0 {
            "⚠️ 轻微损坏"
        } else if self.score >= 50.0 {
            "⚠️ 损坏"
        } else {
            "❌ 严重损坏"
        }
    }
}

/// 修复结果
#[derive(Debug, Clone)]
pub struct RepairResult {
    pub repairs: Vec<String>,
    pub bytes_saved: i64,
}
