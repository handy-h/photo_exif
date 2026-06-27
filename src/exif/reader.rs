use crate::model::{ExifTag, ExifValue};
use anyhow::Result;
use exif::Tag;
use std::collections::HashMap;
use std::path::Path;

/// EXIF 读取器 - 从图片文件中提取 EXIF 数据
pub struct ExifReader;

impl ExifReader {
    /// 读取图片的 EXIF 数据，返回 (entries, 图片格式)
    pub fn read(path: &Path) -> Result<(HashMap<ExifTag, ExifValue>, String)> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        let exif_reader = exif::Reader::new();
        let exif_data = exif_reader.read_from_container(&mut reader)?;

        let mut entries = HashMap::new();

        // 遍历所有 IFD
        for (ifd_name, ifd) in exif_data.entries_for_ifd(exif::Ifd::Primary) {
            for field in ifd {
                let tag = ExifTag::new(
                    field.tag.number(),
                    ifd_name,
                    Self::tag_name(field.tag),
                );

                let value = Self::convert_value(field);
                entries.insert(tag, value);
            }
        }

        // GPS 数据
        if let Ok(gps_ifd) = exif_data.entries_for_ifd(exif::Ifd::GPS) {
            for field in gps_ifd {
                let tag = ExifTag::new(
                    field.tag.number(),
                    "GPS",
                    Self::tag_name(field.tag),
                );
                let value = Self::convert_value(field);
                entries.insert(tag, value);
            }
        }

        // 缩略图 EXIF
        if let Ok(thumb_ifd) = exif_data.entries_for_ifd(exif::Ifd::Thumbnail) {
            for field in thumb_ifd {
                let tag = ExifTag::new(
                    field.tag.number(),
                    "Thumbnail",
                    Self::tag_name(field.tag),
                );
                let value = Self::convert_value(field);
                entries.insert(tag, value);
            }
        }

        let format = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_uppercase();

        Ok((entries, format))
    }

    /// 将 exif crate 的值转换为内部 ExifValue
    fn convert_value(field: &exif::Field) -> ExifValue {
        use exif::Value;

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
                    .filter_map(|s| s.as_ref().map(|s| s.to_string()))
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
            Value::Undefined(v) => ExifValue::Undefined(v.clone()),
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
            Value::Unknown(_, v) => ExifValue::Undefined(v.clone()),
        }
    }

    /// 将 exif crate 的 Tag 转换为显示名
    fn tag_name(tag: Tag) -> String {
        match tag {
            Tag::Make => "制造商".into(),
            Tag::Model => "型号".into(),
            Tag::Orientation => "方向".into(),
            Tag::XResolution => "水平分辨率".into(),
            Tag::YResolution => "垂直分辨率".into(),
            Tag::ResolutionUnit => "分辨率单位".into(),
            Tag::Software => "软件".into(),
            Tag::DateTime => "修改时间".into(),
            Tag::WhitePoint => "白点".into(),
            Tag::PrimaryChromaticities => "主色度".into(),
            Tag::YCbCrCoefficients => "YCbCr系数".into(),
            Tag::YCbCrSubSampling => "YCbCr采样".into(),
            Tag::YCbCrPositioning => "YCbCr定位".into(),
            Tag::ReferenceBlackWhite => "参考黑白".into(),
            Tag::Copyright => "版权".into(),
            Tag::ExposureTime => "曝光时间".into(),
            Tag::FNumber => "光圈值".into(),
            Tag::ExposureProgram => "曝光程序".into(),
            Tag::ISOSpeedRatings => "ISO".into(),
            Tag::ExifVersion => "Exif版本".into(),
            Tag::DateTimeOriginal => "拍摄时间".into(),
            Tag::DateTimeDigitized => "数字化时间".into(),
            Tag::ComponentsConfiguration => "色彩配置".into(),
            Tag::CompressedBitsPerPixel => "压缩率".into(),
            Tag::ShutterSpeedValue => "快门速度".into(),
            Tag::ApertureValue => "光圈".into(),
            Tag::BrightnessValue => "亮度".into(),
            Tag::ExposureBiasValue => "曝光补偿".into(),
            Tag::MaxApertureValue => "最大光圈".into(),
            Tag::SubjectDistance => "拍摄距离".into(),
            Tag::MeteringMode => "测光模式".into(),
            Tag::LightSource => "光源".into(),
            Tag::Flash => "闪光灯".into(),
            Tag::FocalLength => "焦距".into(),
            Tag::SubjectArea => "主体区域".into(),
            Tag::MakerNote => "制造商备注".into(),
            Tag::UserComment => "用户注释".into(),
            Tag::SubsecTime => "亚秒时间".into(),
            Tag::SubsecTimeOriginal => "原始亚秒".into(),
            Tag::SubsecTimeDigitized => "数字化亚秒".into(),
            Tag::FlashpixVersion => "Flashpix版本".into(),
            Tag::ColorSpace => "颜色空间".into(),
            Tag::PixelXDimension => "像素宽度".into(),
            Tag::PixelYDimension => "像素高度".into(),
            Tag::FocalPlaneXResolution => "焦平面X分辨率".into(),
            Tag::FocalPlaneYResolution => "焦平面Y分辨率".into(),
            Tag::FocalPlaneResolutionUnit => "焦平面单位".into(),
            Tag::SensingMethod => "感应方式".into(),
            Tag::FileSource => "文件来源".into(),
            Tag::SceneType => "场景类型".into(),
            Tag::CFAPattern => "CFA模式".into(),
            Tag::CustomRendered => "自定义渲染".into(),
            Tag::ExposureMode => "曝光模式".into(),
            Tag::WhiteBalance => "白平衡".into(),
            Tag::DigitalZoomRatio => "数码变焦".into(),
            Tag::FocalLengthIn35mmFilm => "35mm等效焦距".into(),
            Tag::SceneCaptureType => "场景捕获类型".into(),
            Tag::GainControl => "增益控制".into(),
            Tag::Contrast => "对比度".into(),
            Tag::Saturation => "饱和度".into(),
            Tag::Sharpness => "锐度".into(),
            Tag::DeviceSettingDescription => "设备设置".into(),
            Tag::SubjectDistanceRange => "距离范围".into(),
            Tag::LensSpecification => "镜头规格".into(),
            // GPS
            Tag::GPSVersionID => "GPS版本".into(),
            Tag::GPSLatitudeRef => "纬度方向".into(),
            Tag::GPSLatitude => "纬度".into(),
            Tag::GPSLongitudeRef => "经度方向".into(),
            Tag::GPSLongitude => "经度".into(),
            Tag::GPSAltitudeRef => "高度参考".into(),
            Tag::GPSAltitude => "高度".into(),
            Tag::GPSTimeStamp => "GPS时间".into(),
            Tag::GPSSatellites => "卫星数".into(),
            Tag::GPSStatus => "GPS状态".into(),
            Tag::GPSMeasureMode => "测量模式".into(),
            Tag::GPSDOP => "精度".into(),
            Tag::GPSImgDirectionRef => "方向参考".into(),
            Tag::GPSImgDirection => "方向".into(),
            Tag::GPSMapDatum => "大地基准".into(),
            Tag::GPSDestLatitude => "目标纬度".into(),
            Tag::GPSDestLongitude => "目标经度".into(),
            Tag::GPSDestBearingRef => "目标方向参考".into(),
            Tag::GPSDestBearing => "目标方向".into(),
            Tag::GPSDestDistance => "目标距离".into(),
            Tag::GPSProcessingMethod => "处理方法".into(),
            Tag::GPSAreaInformation => "区域信息".into(),
            Tag::GPSDateStamp => "GPS日期".into(),
            Tag::GPSDifferential => "差分".into(),
            // Thumbnail
            Tag::ThumbnailImageWidth => "缩略图宽度".into(),
            Tag::ThumbnailImageHeight => "缩略图高度".into(),
            Tag::ThumbnailBitsPerSample => "缩略图位深".into(),
            Tag::ThumbnailCompression => "缩略图压缩".into(),
            Tag::ThumbnailPhotometricInterpretation => "缩略图色彩".into(),
            Tag::ThumbnailOrientation => "缩略图方向".into(),
            Tag::ThumbnailSamplesPerPixel => "缩略图采样".into(),
            Tag::ThumbnailRowsPerStrip => "缩略图行".into(),
            Tag::ThumbnailStripBytesCount => "缩略图字节".into(),
            Tag::ThumbnailXResolution => "缩略图X分辨率".into(),
            Tag::ThumbnailYResolution => "缩略图Y分辨率".into(),
            Tag::ThumbnailPlanarConfiguration => "缩略图平面".into(),
            Tag::ThumbnailResolutionUnit => "缩略图单位".into(),
            // 其他常见 tag
            Tag::ImageWidth => "图像宽度".into(),
            Tag::ImageLength => "图像高度".into(),
            Tag::BitsPerSample => "位深度".into(),
            Tag::Compression => "压缩方式".into(),
            Tag::PhotometricInterpretation => "色彩解释".into(),
            Tag::StripOffsets => "条偏移".into(),
            Tag::SamplesPerPixel => "采样/像素".into(),
            Tag::RowsPerStrip => "行/条".into(),
            Tag::StripByteCounts => "条字节数".into(),
            Tag::PlanarConfiguration => "平面配置".into(),
            Tag::TileWidth => "瓦片宽度".into(),
            Tag::TileLength => "瓦片长度".into(),
            Tag::TileOffset => "瓦片偏移".into(),
            Tag::TileByteCounts => "瓦片字节数".into(),
            _ => {
                // 尝试获取 tag 名称，失败则用 hex
                let name = format!("{:?}", tag);
                if name.starts_with("Unknown") {
                    format!("Tag 0x{:04X}", tag.number())
                } else {
                    name
                }
            }
        }
    }
}
