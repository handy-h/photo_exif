use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

/// GPX 轨迹点
#[derive(Debug, Clone)]
pub struct GpxPoint {
    pub time: DateTime<Utc>,
    pub lat: f64,
    pub lon: f64,
    pub ele: Option<f64>,
}

/// GPX 文件解析器
pub struct GpxParser;

impl GpxParser {
    /// 解析 GPX 文件，返回按时间排序的轨迹点
    pub fn parse(path: &Path) -> Result<Vec<GpxPoint>> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("无法读取 GPX 文件: {}", path.display()))?;

        Self::parse_str(&content)
    }

    /// 解析 GPX 字符串
    pub fn parse_str(content: &str) -> Result<Vec<GpxPoint>> {
        let mut reader = Reader::from_str(content);
        reader.trim_text(true);

        let mut points = Vec::new();
        let mut buf = Vec::new();
        let mut current_lat: Option<f64> = None;
        let mut current_lon: Option<f64> = None;
        let mut current_ele: Option<f64> = None;
        let mut current_time: Option<DateTime<Utc>> = None;
        let mut in_trkpt = false;
        let mut in_ele = false;
        let mut in_time = false;
        let mut text_buf = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => anyhow::bail!("GPX 解析错误: {}", e),
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    let name_bytes = e.name().as_ref().to_vec();
                    let name = String::from_utf8_lossy(&name_bytes);
                    match name.as_ref() {
                        "trkpt" | "wpt" | "rtept" => {
                            in_trkpt = true;
                            current_lat = None;
                            current_lon = None;
                            current_ele = None;
                            current_time = None;

                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref());
                                let val = String::from_utf8_lossy(&attr.value);
                                match key.as_ref() {
                                    "lat" => current_lat = val.parse().ok(),
                                    "lon" => current_lon = val.parse().ok(),
                                    _ => {}
                                }
                            }
                        }
                        "ele" if in_trkpt => {
                            in_ele = true;
                            text_buf.clear();
                        }
                        "time" if in_trkpt => {
                            in_time = true;
                            text_buf.clear();
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_ele || in_time {
                        text_buf.push_str(&e.unescape()?);
                    }
                }
                Ok(Event::End(e)) => {
                    let name_bytes = e.name().as_ref().to_vec();
                    let name = String::from_utf8_lossy(&name_bytes);
                    match name.as_ref() {
                        "ele" if in_trkpt => {
                            current_ele = text_buf.trim().parse().ok();
                            in_ele = false;
                        }
                        "time" if in_trkpt => {
                            let t = text_buf.trim();
                            current_time = Self::parse_time(t);
                            in_time = false;
                        }
                        "trkpt" | "wpt" | "rtept" => {
                            if let (Some(lat), Some(lon), Some(time)) =
                                (current_lat, current_lon, current_time)
                            {
                                points.push(GpxPoint {
                                    time,
                                    lat,
                                    lon,
                                    ele: current_ele,
                                });
                            }
                            in_trkpt = false;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            buf.clear();
        }

        // 按时间排序
        points.sort_by_key(|p| p.time);
        Ok(points)
    }

    /// 解析 GPX 时间格式 (ISO 8601)
    fn parse_time(s: &str) -> Option<DateTime<Utc>> {
        // 尝试标准 ISO 8601
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Some(dt.with_timezone(&Utc));
        }
        // 尝试不带时区后缀的格式: "2024-01-01T12:00:00Z"
        if let Some(stripped) = s.strip_suffix('Z') {
            if let Ok(ndt) = NaiveDateTime::parse_from_str(stripped, "%Y-%m-%dT%H:%M:%S") {
                return Some(ndt.and_utc());
            }
            // 带小数秒
            if let Ok(ndt) = NaiveDateTime::parse_from_str(stripped, "%Y-%m-%dT%H:%M:%S%.f") {
                return Some(ndt.and_utc());
            }
        }
        // 尝试不带 Z
        if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
            return Some(ndt.and_utc());
        }
        if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f") {
            return Some(ndt.and_utc());
        }
        None
    }

    /// 根据拍摄时间匹配最近的 GPX 轨迹点
    /// photo_time: 拍摄时间 (ISO 格式字符串，如 "2024:01:01 12:00:00")
    /// 返回 (lat, lon, ele) 十进制坐标
    pub fn match_point(
        points: &[GpxPoint],
        photo_time: &str,
    ) -> Option<(f64, f64, Option<f64>)> {
        // 解析拍摄时间 (EXIF 格式: "2024:01:01 12:00:00")
        let pt = Self::parse_exif_time(photo_time)?;

        if points.is_empty() {
            return None;
        }

        // 二分查找最近的时间点
        let idx = points
            .binary_search_by_key(&pt, |p| p.time)
            .unwrap_or_else(|i| i);

        // 比较 idx-1 和 idx，选更近的
        let candidate = if idx == 0 {
            &points[0]
        } else if idx >= points.len() {
            &points[points.len() - 1]
        } else {
            let prev = &points[idx - 1];
            let next = &points[idx];
            let diff_prev = (prev.time - pt).num_seconds().abs();
            let diff_next = (next.time - pt).num_seconds().abs();
            if diff_prev <= diff_next {
                prev
            } else {
                next
            }
        };

        // 时间差超过 5 分钟则认为不匹配
        let diff = (candidate.time - pt).num_seconds().abs();
        if diff > 300 {
            return None;
        }

        Some((candidate.lat, candidate.lon, candidate.ele))
    }

    /// 解析 EXIF 时间格式 "2024:01:01 12:00:00" 或 "2024-01-01T12:00:00"
    fn parse_exif_time(s: &str) -> Option<DateTime<Utc>> {
        let s = s.trim().trim_end_matches('\0');

        // EXIF 标准格式: "2024:01:01 12:00:00"
        if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S") {
            return Some(ndt.and_utc());
        }

        // 带小数秒
        if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S%.f") {
            return Some(ndt.and_utc());
        }

        // ISO 格式
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Some(dt.with_timezone(&Utc));
        }

        // "2024-01-01 12:00:00"
        if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
            return Some(ndt.and_utc());
        }

        None
    }

    /// 将十进制经纬度转换为 EXIF 度分秒格式
    /// 返回 (degrees, minutes, seconds) 作为 Rational
    pub fn decimal_to_dms(decimal: f64) -> (u32, u32, u32) {
        let abs_val = decimal.abs();
        let degrees = abs_val.floor() as u32;
        let minutes_full = (abs_val - degrees as f64) * 60.0;
        let minutes = minutes_full.floor() as u32;
        let seconds = ((minutes_full - minutes as f64) * 60.0).round() as u32;
        (degrees, minutes, seconds)
    }
}

/// GPX 匹配器 — 给一组照片批量匹配 GPS
#[derive(Debug)]
pub struct GpxMatcher {
    points: Vec<GpxPoint>,
}

impl GpxMatcher {
    pub fn new(points: Vec<GpxPoint>) -> Self {
        Self { points }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let points = GpxParser::parse(path)?;
        Ok(Self { points })
    }

    /// 为单张照片匹配 GPS
    /// photo_time: 拍摄时间字符串
    /// 返回 (lat_dms, lat_ref, lon_dms, lon_ref, ele)
    pub fn match_photo(
        &self,
        photo_time: &str,
    ) -> Option<((u32, u32, u32), char, (u32, u32, u32), char, Option<f64>)> {
        let (lat, lon, ele) = GpxParser::match_point(&self.points, photo_time)?;

        let lat_dms = GpxParser::decimal_to_dms(lat);
        let lat_ref = if lat >= 0.0 { 'N' } else { 'S' };

        let lon_dms = GpxParser::decimal_to_dms(lon);
        let lon_ref = if lon >= 0.0 { 'E' } else { 'W' };

        Some((lat_dms, lat_ref, lon_dms, lon_ref, ele))
    }

    /// 获取轨迹点数量
    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    /// 获取轨迹时间范围
    pub fn time_range(&self) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        if self.points.is_empty() {
            return None;
        }
        Some((self.points[0].time, self.points[self.points.len() - 1].time))
    }
}
