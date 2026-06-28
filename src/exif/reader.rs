use crate::model::{ExifTag, ExifValue};
use anyhow::Result;
use exif::{In, Tag, Value};
use std::collections::HashMap;
use std::path::Path;

/// EXIF 读取器 - 从图片文件中提取 EXIF 数据
pub struct ExifReader;

impl ExifReader {
    /// 读取图片的 EXIF 数据，返回 (entries, 图片格式)
    pub fn read(path: &Path) -> Result<(HashMap<ExifTag, ExifValue>, String)> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        let mut exif_reader = exif::Reader::new();
        let exif_reader = exif_reader.continue_on_error(true);
        let exif_data = exif_reader.read_from_container(&mut reader)?;

        let mut entries = HashMap::new();

        for field in exif_data.fields() {
            let ifd_name = Self::ifd_name(field.ifd_num);
            let tag_name = Self::tag_name(field.tag);
            let tag = ExifTag::new(field.tag.number(), &ifd_name, &tag_name);
            let value = Self::convert_value(field);
            entries.insert(tag, value);
        }

        let format = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_uppercase();

        Ok((entries, format))
    }

    fn ifd_name(in_: In) -> String {
        match in_.index() {
            0 => "Primary".to_string(),
            1 => "Thumbnail".to_string(),
            2 => "ExifIFD".to_string(),
            3 => "GPS".to_string(),
            4 => "InteropIFD".to_string(),
            _ => format!("IFD{}", in_.index()),
        }
    }

    /// 将 exif crate 的值转换为内部 ExifValue
    fn convert_value(field: &exif::Field) -> ExifValue {
        match &field.value {
            Value::Byte(v) => {
                if v.len() == 1 {
                    ExifValue::Byte(v.clone())
                } else {
                    ExifValue::Slice(v.clone())
                }
            }
            Value::Ascii(v) => ExifValue::Ascii(
                v.iter()
                    .filter_map(|s| std::str::from_utf8(s).ok().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            Value::Short(v) => {
                if v.len() == 1 {
                    ExifValue::Short(v[0])
                } else {
                    ExifValue::Slice(v.iter().flat_map(|&n| n.to_le_bytes()).collect())
                }
            }
            Value::Long(v) => {
                if v.len() == 1 {
                    ExifValue::Long(v[0])
                } else {
                    ExifValue::Slice(v.iter().flat_map(|&n| n.to_le_bytes()).collect())
                }
            }
            Value::Rational(v) => {
                if let Some(r) = v.first() {
                    ExifValue::Rational(r.num, r.denom)
                } else {
                    ExifValue::Slice(vec![])
                }
            }
            Value::SRational(v) => {
                if let Some(r) = v.first() {
                    ExifValue::SRational(r.num, r.denom)
                } else {
                    ExifValue::Slice(vec![])
                }
            }
            Value::Undefined(v, _) => ExifValue::Undefined(v.clone()),
            Value::SByte(v) => ExifValue::Slice(v.iter().map(|&b| b as u8).collect()),
            Value::SShort(v) => ExifValue::Slice(
                v.iter()
                    .flat_map(|&n| n.to_le_bytes())
                    .map(|b| b as u8)
                    .collect(),
            ),
            Value::SLong(v) => ExifValue::Slice(
                v.iter()
                    .flat_map(|&n| n.to_le_bytes())
                    .map(|b| b as u8)
                    .collect(),
            ),
            Value::Float(v) => {
                if let Some(f) = v.first() {
                    let bytes = f.to_le_bytes();
                    ExifValue::Slice(bytes.to_vec())
                } else {
                    ExifValue::Slice(vec![])
                }
            }
            Value::Double(v) => {
                if let Some(d) = v.first() {
                    let bytes = d.to_le_bytes();
                    ExifValue::Slice(bytes.to_vec())
                } else {
                    ExifValue::Slice(vec![])
                }
            }
            Value::Unknown(_, _, _) => ExifValue::Undefined(vec![]),
        }
    }

    /// 将 exif crate 的 Tag 转换为显示名
    fn tag_name(tag: Tag) -> String {
        // 先尝试用 tag.description()
        if let Some(desc) = tag.description() {
            return desc.to_string();
        }

        // 内置中文映射
        match tag {
            Tag::Make => "制造商".to_string(),
            Tag::Model => "型号".to_string(),
            Tag::Orientation => "方向".to_string(),
            Tag::XResolution => "水平分辨率".to_string(),
            Tag::YResolution => "垂直分辨率".to_string(),
            Tag::ResolutionUnit => "分辨率单位".to_string(),
            Tag::Software => "软件".to_string(),
            Tag::DateTime => "修改时间".to_string(),
            Tag::Copyright => "版权".to_string(),
            Tag::ExposureTime => "曝光时间".to_string(),
            Tag::FNumber => "光圈值".to_string(),
            Tag::ExposureProgram => "曝光程序".to_string(),
            Tag::ISOSpeed => "ISO".to_string(),
            Tag::ExifVersion => "Exif版本".to_string(),
            Tag::DateTimeOriginal => "拍摄时间".to_string(),
            Tag::DateTimeDigitized => "数字化时间".to_string(),
            Tag::ShutterSpeedValue => "快门速度".to_string(),
            Tag::ApertureValue => "光圈".to_string(),
            Tag::BrightnessValue => "亮度".to_string(),
            Tag::ExposureBiasValue => "曝光补偿".to_string(),
            Tag::MaxApertureValue => "最大光圈".to_string(),
            Tag::SubjectDistance => "拍摄距离".to_string(),
            Tag::MeteringMode => "测光模式".to_string(),
            Tag::LightSource => "光源".to_string(),
            Tag::Flash => "闪光灯".to_string(),
            Tag::FocalLength => "焦距".to_string(),
            Tag::SubjectArea => "主体区域".to_string(),
            Tag::MakerNote => "制造商备注".to_string(),
            Tag::UserComment => "用户注释".to_string(),
            Tag::SubSecTime => "亚秒时间".to_string(),
            Tag::SubSecTimeOriginal => "原始亚秒".to_string(),
            Tag::SubSecTimeDigitized => "数字化亚秒".to_string(),
            Tag::FlashpixVersion => "Flashpix版本".to_string(),
            Tag::ColorSpace => "颜色空间".to_string(),
            Tag::PixelXDimension => "像素宽度".to_string(),
            Tag::PixelYDimension => "像素高度".to_string(),
            Tag::FocalPlaneXResolution => "焦平面X分辨率".to_string(),
            Tag::FocalPlaneYResolution => "焦平面Y分辨率".to_string(),
            Tag::FocalPlaneResolutionUnit => "焦平面单位".to_string(),
            Tag::SensingMethod => "感应方式".to_string(),
            Tag::FileSource => "文件来源".to_string(),
            Tag::SceneType => "场景类型".to_string(),
            Tag::CFAPattern => "CFA模式".to_string(),
            Tag::CustomRendered => "自定义渲染".to_string(),
            Tag::ExposureMode => "曝光模式".to_string(),
            Tag::WhiteBalance => "白平衡".to_string(),
            Tag::DigitalZoomRatio => "数码变焦".to_string(),
            Tag::FocalLengthIn35mmFilm => "35mm等效焦距".to_string(),
            Tag::SceneCaptureType => "场景捕获类型".to_string(),
            Tag::GainControl => "增益控制".to_string(),
            Tag::Contrast => "对比度".to_string(),
            Tag::Saturation => "饱和度".to_string(),
            Tag::Sharpness => "锐度".to_string(),
            Tag::DeviceSettingDescription => "设备设置".to_string(),
            Tag::SubjectDistanceRange => "距离范围".to_string(),
            Tag::LensSpecification => "镜头规格".to_string(),
            Tag::LensMake => "镜头制造商".to_string(),
            Tag::LensModel => "镜头型号".to_string(),
            Tag::LensSerialNumber => "镜头序列号".to_string(),
            // GPS
            Tag::GPSVersionID => "GPS版本".to_string(),
            Tag::GPSLatitudeRef => "纬度方向".to_string(),
            Tag::GPSLatitude => "纬度".to_string(),
            Tag::GPSLongitudeRef => "经度方向".to_string(),
            Tag::GPSLongitude => "经度".to_string(),
            Tag::GPSAltitudeRef => "高度参考".to_string(),
            Tag::GPSAltitude => "高度".to_string(),
            Tag::GPSTimeStamp => "GPS时间".to_string(),
            Tag::GPSSatellites => "卫星数".to_string(),
            Tag::GPSStatus => "GPS状态".to_string(),
            Tag::GPSMeasureMode => "测量模式".to_string(),
            Tag::GPSDOP => "精度".to_string(),
            Tag::GPSSpeedRef => "速度单位".to_string(),
            Tag::GPSSpeed => "速度".to_string(),
            Tag::GPSTrackRef => "方向参考".to_string(),
            Tag::GPSTrack => "方向".to_string(),
            Tag::GPSImgDirectionRef => "图片方向参考".to_string(),
            Tag::GPSImgDirection => "图片方向".to_string(),
            Tag::GPSMapDatum => "大地基准".to_string(),
            Tag::GPSDestLatitudeRef => "目标纬度方向".to_string(),
            Tag::GPSDestLatitude => "目标纬度".to_string(),
            Tag::GPSDestLongitudeRef => "目标经度方向".to_string(),
            Tag::GPSDestLongitude => "目标经度".to_string(),
            Tag::GPSDestBearingRef => "目标方向参考".to_string(),
            Tag::GPSDestBearing => "目标方向".to_string(),
            Tag::GPSDestDistanceRef => "目标距离单位".to_string(),
            Tag::GPSDestDistance => "目标距离".to_string(),
            Tag::GPSProcessingMethod => "处理方法".to_string(),
            Tag::GPSAreaInformation => "区域信息".to_string(),
            Tag::GPSDateStamp => "GPS日期".to_string(),
            Tag::GPSDifferential => "差分".to_string(),
            // 其他常见
            Tag::ImageWidth => "图像宽度".to_string(),
            Tag::ImageLength => "图像高度".to_string(),
            Tag::BitsPerSample => "位深度".to_string(),
            Tag::Compression => "压缩方式".to_string(),
            Tag::PhotometricInterpretation => "色彩解释".to_string(),
            Tag::StripOffsets => "条偏移".to_string(),
            Tag::SamplesPerPixel => "采样/像素".to_string(),
            Tag::RowsPerStrip => "行/条".to_string(),
            Tag::StripByteCounts => "条字节数".to_string(),
            Tag::PlanarConfiguration => "平面配置".to_string(),
            _ => {
                format!("Tag 0x{:04X}", tag.number())
            }
        }
    }
}
