use crate::model::ExifValue;

/// EXIF 值格式化器 - 将原始值转换为更友好的显示格式
pub struct ExifFormatter;

impl ExifFormatter {
    /// 智能格式化 EXIF 值
    pub fn format(tag_id: u16, value: &ExifValue) -> String {
        let raw = value.to_display_string();

        match tag_id {
            // 曝光时间：显示为 "1/500s" 格式
            0x829A => Self::format_exposure_time(value),
            // 光圈值：显示为 "f/2.8" 格式
            0x829D => Self::format_aperture(value),
            // ISO
            0x8827 => raw,
            // 焦距：显示为 "50mm" 格式
            0x920A => Self::format_focal_length(value),
            // 曝光补偿：显示为 "+0.3EV" 格式
            0x9204 => Self::format_exposure_bias(value),
            // GPS 坐标：显示为 DMS 格式
            0x0002 | 0x0004 => Self::format_gps_coord(value),
            // 分辨率：显示为 "72 DPI" 格式
            0x011A | 0x011B => Self::format_resolution(value),
            // 默认：直接使用原始格式化
            _ => raw,
        }
    }

    /// 格式化曝光时间
    fn format_exposure_time(value: &ExifValue) -> String {
        match value {
            ExifValue::Rational(n, d) => {
                if *d == 0 {
                    return "0s".to_string();
                }
                if *n == 1 {
                    format!("1/{}s", d)
                } else if *n > *d {
                    format!("{:.1}s", *n as f64 / *d as f64)
                } else {
                    format!("{}/{}s", n, d)
                }
            }
            _ => value.to_display_string(),
        }
    }

    /// 格式化光圈值
    fn format_aperture(value: &ExifValue) -> String {
        match value {
            ExifValue::Rational(n, d) => {
                if *d == 0 {
                    return "f/0".to_string();
                }
                let f_number = (*n as f64 / *d as f64).sqrt();
                format!("f/{:.1}", f_number)
            }
            _ => value.to_display_string(),
        }
    }

    /// 格式化焦距
    fn format_focal_length(value: &ExifValue) -> String {
        match value {
            ExifValue::Rational(n, d) => {
                if *d == 0 {
                    return "0mm".to_string();
                }
                let mm = *n as f64 / *d as f64;
                if mm.fract() == 0.0 {
                    format!("{:.0}mm", mm)
                } else {
                    format!("{:.1}mm", mm)
                }
            }
            _ => value.to_display_string(),
        }
    }

    /// 格式化曝光补偿
    fn format_exposure_bias(value: &ExifValue) -> String {
        match value {
            ExifValue::Rational(n, d) => {
                if *d == 0 {
                    return "0EV".to_string();
                }
                let ev = *n as f64 / *d as f64;
                let sign = if ev >= 0.0 { "+" } else { "" };
                format!("{}{:.1}EV", sign, ev)
            }
            _ => value.to_display_string(),
        }
    }

    /// 格式化 GPS 坐标（度分秒转十进制）
    pub fn format_gps_dms(d: u32, m: u32, s: u32, ref_str: &str) -> String {
        let decimal = d as f64 + (m as f64) / 60.0 + (s as f64) / 3600.0;
        let sign = if ref_str == "S" || ref_str == "W" { -1.0 } else { 1.0 };
        format!("{:.6}°{}", decimal * sign, ref_str)
    }

    /// 格式化 GPS 坐标（从原始值解析）
    fn format_gps_coord(value: &ExifValue) -> String {
        match value {
            ExifValue::Rational(n, d) => {
                if *d == 0 {
                    return "0°".to_string();
                }
                let total = *n as f64 / *d as f64;
                let degrees = total.floor() as u32;
                let minutes = ((total - degrees as f64) * 60.0).floor() as u32;
                let seconds = ((total - degrees as f64 - minutes as f64 / 60.0) * 3600.0).round() as u32;
                format!("{}°{}'{}\"", degrees, minutes, seconds)
            }
            _ => value.to_display_string(),
        }
    }

    /// 格式化分辨率
    fn format_resolution(value: &ExifValue) -> String {
        match value {
            ExifValue::Rational(n, d) => {
                if *d == 0 {
                    return "0".to_string();
                }
                let res = *n as f64 / *d as f64;
                format!("{:.0} DPI", res)
            }
            _ => value.to_display_string(),
        }
    }
}
