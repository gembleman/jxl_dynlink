use std::{
    io::Cursor,
    os::raw::{c_char, c_void},
    path::PathBuf,
};

use image::{
    codecs::png::PngDecoder, ColorType, DynamicImage, GenericImageView, ImageDecoder, ImageFormat,
};

use crate::{
    JxlBasicInfo, JxlColorEncoding, JxlEncoder, JxlEncoderFrameSettingId, JxlEncoderStatus,
    JxlError,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
/// jxl_bool 열거형 정의
pub enum JxlBool {
    /// JXL_FALSE
    False = 0,
    /// JXL_TRUE
    True = 1,
}

impl Default for JxlBool {
    fn default() -> Self {
        JxlBool::False
    }
}

impl From<bool> for JxlBool {
    fn from(b: bool) -> Self {
        if b {
            JxlBool::True
        } else {
            JxlBool::False
        }
    }
}

impl From<JxlBool> for bool {
    fn from(jxl_bool: JxlBool) -> Self {
        jxl_bool == JxlBool::True
    }
}

/// 메모리 관리 함수 타입 정의
pub type JpegxlAllocFunc = unsafe extern "C" fn(*mut c_void, usize) -> *mut c_void;
pub type JpegxlFreeFunc = unsafe extern "C" fn(*mut c_void, *mut c_void);

/// JxlMemoryManager 구조체 정의
#[repr(C)]
pub struct JxlMemoryManager {
    pub opaque: *mut c_void,
    pub alloc: Option<JpegxlAllocFunc>,
    pub free: Option<JpegxlFreeFunc>,
}

/// JxlBoxType 정의
pub type JxlBoxType = [c_char; 4];

/// JxlDataType 열거형 정의
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlDataType {
    ///JXL_TYPE_FLOAT
    Float = 0,
    ///JXL_TYPE_UINT8
    Uint8 = 2,
    ///JXL_TYPE_UINT16
    Uint16 = 3,
    ///JXL_TYPE_FLOAT16
    Float16 = 5,
}

/// JxlEndianness 열거형 정의
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlEndianness {
    ///JXL_NATIVE_ENDIAN
    NativeEndian = 0,
    ///JXL_LITTLE_ENDIAN
    LittleEndian = 1,
    ///JXL_BIG_ENDIAN
    BigEndian = 2,
}

/// JxlBitDepthType 열거형 정의
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlBitDepthType {
    ///JXL_BIT_DEPTH_FROM_PIXEL_FORMAT
    FromPixelFormat = 0,
    ///JXL_BIT_DEPTH_FROM_CODESTREAM
    FromCodestream = 1,
    ///JXL_BIT_DEPTH_CUSTOM
    Custom = 2,
}

/// JxlPixelFormat 구조체 정의
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlPixelFormat {
    pub num_channels: u32,
    /// Data type of each channel.
    pub data_type: JxlDataType,
    /// Whether multi-byte data types are represented in big endian or little endian format. This applies to JXL_TYPE_UINT16 and JXL_TYPE_FLOAT.
    pub endianness: JxlEndianness,
    /// Align scanlines to a multiple of align bytes, or 0 to require no alignment at all (which has the same effect as value 1)
    pub align: usize,
}
impl Default for JxlPixelFormat {
    fn default() -> Self {
        JxlPixelFormat {
            num_channels: 3,
            data_type: JxlDataType::Uint8,
            endianness: JxlEndianness::NativeEndian,
            align: 0,
        }
    }
}

// JxlBitDepth 구조체 정의
/// 허용되는 입력 및 출력 픽셀 값의 범위에 따라 입력 및 출력 버퍼의 해석을 설명하는 데이터 유형입니다.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlBitDepth {
    pub type_: JxlBitDepthType,
    /// Custom bits per sample
    /// 사용자 정의 샘플 비트
    pub bits_per_sample: u32,
    /// Custom exponent bits per sample
    /// 사용자 정의 지수 비트
    pub exponent_bits_per_sample: u32,
}
// JxlBitDepth의 기본 구현
impl Default for JxlBitDepth {
    fn default() -> Self {
        JxlBitDepth {
            type_: JxlBitDepthType::FromPixelFormat,
            bits_per_sample: 8,
            exponent_bits_per_sample: 0,
        }
    }
}

// 유틸리티 함수
#[inline]
pub fn to_jxl_bool(value: bool) -> JxlBool {
    if value {
        JxlBool::True
    } else {
        JxlBool::False
    }
}

#[inline]
pub fn from_jxl_bool(value: JxlBool) -> bool {
    value != JxlBool::False
}

// JxlMemoryManager의 기본 구현
impl Default for JxlMemoryManager {
    fn default() -> Self {
        JxlMemoryManager {
            opaque: std::ptr::null_mut(),
            alloc: None,
            free: None,
        }
    }
}

const INITIAL_BUFFER_SIZE: usize = 1024 * 1024;

// JxlBoxType의 기본 구현
pub fn jpg_to_lossless_jxl(
    dll_path: &PathBuf,
    input_data: &[u8],
    effort: i64,
) -> Result<Vec<u8>, JxlError> {
    // JxlEncoder 생성
    let mut encoder = JxlEncoder::new(dll_path, None)?;

    // 프레임 설정 생성 및 옵션 설정
    let frame_settings = encoder.create_frame_settings(None)?;
    encoder.set_frame_option(frame_settings, JxlEncoderFrameSettingId::Effort, effort)?;

    // JPEG 프레임 추가
    encoder.add_jpeg_frame(frame_settings, input_data)?;

    // 입력 종료
    encoder.close_input()?;

    // 출력 데이터 처리
    let mut compressed = Vec::new();

    let mut buffer = vec![0u8; INITIAL_BUFFER_SIZE]; // 초기 버퍼 크기 설정

    loop {
        let mut next_out = buffer.as_mut_ptr();
        let mut avail_out = buffer.len();

        let status = encoder.process_output(&mut next_out, &mut avail_out)?;
        let bytes_written = buffer.len() - avail_out;
        compressed.extend_from_slice(&buffer[..bytes_written]);

        if status == JxlEncoderStatus::Success {
            break;
        } else if status == JxlEncoderStatus::NeedMoreOutput {
            // 버퍼 크기를 늘려야 할 경우
            buffer.resize(buffer.len() * 2, 0);
        } else {
            return Err(JxlError::EncodingFailed(status));
        }
    }

    Ok(compressed)
}

pub fn png_to_lossless_jxl(
    dll_path: &PathBuf,
    img_data: &[u8],
    effort: i64,
    distance: f32,
) -> Result<Vec<u8>, JxlError> {
    // 이미지 데이터를 메모리에서 로드
    let img = image::load_from_memory_with_format(img_data, ImageFormat::Png)
        .map_err(|e| JxlError::InvalidInput(e.to_string()))?;
    let (width, height) = img.dimensions();

    // PNG 디코더 생성하여 ICC 프로파일 추출
    let cursor = Cursor::new(img_data);
    let mut decoder = PngDecoder::new(cursor).expect("Failed to create PNG decoder");

    // JxlEncoder 생성
    let mut encoder = JxlEncoder::new(dll_path, None)?;

    // 기본 정보 설정
    let mut basic_info = JxlBasicInfo::default();
    encoder.init_basic_info(&mut basic_info)?;
    basic_info.xsize = width;
    basic_info.ysize = height;

    // 색상 정보 설정
    let (num_color_channels, bits_per_sample, alpha_bits) = match img.color() {
        ColorType::L8 => (1, 8, 0),
        ColorType::La8 => (1, 8, 8),
        ColorType::Rgb8 => (3, 8, 0),
        ColorType::Rgba8 => (3, 8, 8),
        ColorType::L16 => (1, 16, 0),
        ColorType::La16 => (1, 16, 16),
        ColorType::Rgb16 => (3, 16, 0),
        ColorType::Rgba16 => (3, 16, 16),
        _ => {
            return Err(JxlError::UnsupportedOperation(
                "Unsupported color type".to_string(),
            ))
        }
    };

    basic_info.num_color_channels = num_color_channels;
    basic_info.bits_per_sample = bits_per_sample;
    basic_info.alpha_bits = alpha_bits;
    basic_info.num_extra_channels = if alpha_bits > 0 { 1 } else { 0 };

    encoder.set_basic_info(&basic_info)?;

    // 색상 인코딩 설정 (ICC 프로파일 또는 sRGB)
    if let Some(icc_profile) = decoder.icc_profile().expect("Failed to get ICC profile") {
        encoder.set_icc_profile(&icc_profile)?;
    } else {
        let mut color_encoding = JxlColorEncoding::default();
        encoder.color_encoding_set_to_srgb(&mut color_encoding, num_color_channels == 1)?;
        encoder.set_color_encoding(&color_encoding)?;
    }

    // 프레임 설정 생성
    let frame_settings = encoder.create_frame_settings(None)?;

    // 프레임 옵션 설정
    encoder.set_frame_option(frame_settings, JxlEncoderFrameSettingId::Effort, effort)?; // effort = 7
    encoder.set_frame_distance(frame_settings, distance)?; //distance = 0.0

    let bit_depth = JxlBitDepth::default();
    encoder.set_frame_bit_depth(frame_settings, &bit_depth)?;

    // 픽셀 포맷 및 픽셀 데이터 준비
    let (pixel_format, pixels) = match img {
        DynamicImage::ImageLuma8(_) => (
            JxlPixelFormat {
                num_channels: 1,
                data_type: JxlDataType::Uint8,
                endianness: JxlEndianness::NativeEndian,
                align: 0,
            },
            img.to_luma8().into_raw(),
        ),
        DynamicImage::ImageLumaA8(_) => (
            JxlPixelFormat {
                num_channels: 2,
                data_type: JxlDataType::Uint8,
                endianness: JxlEndianness::NativeEndian,
                align: 0,
            },
            img.to_luma_alpha8().into_raw(),
        ),
        DynamicImage::ImageRgb8(_) => (
            JxlPixelFormat {
                num_channels: 3,
                data_type: JxlDataType::Uint8,
                endianness: JxlEndianness::NativeEndian,
                align: 0,
            },
            img.to_rgb8().into_raw(),
        ),
        DynamicImage::ImageRgba8(_) => (
            JxlPixelFormat {
                num_channels: 4,
                data_type: JxlDataType::Uint8,
                endianness: JxlEndianness::NativeEndian,
                align: 0,
            },
            img.to_rgba8().into_raw(),
        ),
        DynamicImage::ImageLuma16(_) => (
            JxlPixelFormat {
                num_channels: 1,
                data_type: JxlDataType::Uint16,
                endianness: JxlEndianness::NativeEndian,
                align: 0,
            },
            img.to_luma16()
                .as_raw()
                .iter()
                .flat_map(|&pixel| pixel.to_ne_bytes())
                .collect(),
        ),
        DynamicImage::ImageLumaA16(_) => (
            JxlPixelFormat {
                num_channels: 2,
                data_type: JxlDataType::Uint16,
                endianness: JxlEndianness::NativeEndian,
                align: 0,
            },
            img.to_luma_alpha16()
                .as_raw()
                .iter()
                .flat_map(|&pixel| pixel.to_ne_bytes())
                .collect(),
        ),
        DynamicImage::ImageRgb16(_) => (
            JxlPixelFormat {
                num_channels: 3,
                data_type: JxlDataType::Uint16,
                endianness: JxlEndianness::NativeEndian,
                align: 0,
            },
            img.to_rgb16()
                .as_raw()
                .iter()
                .flat_map(|&pixel| pixel.to_ne_bytes())
                .collect(),
        ),
        DynamicImage::ImageRgba16(_) => (
            JxlPixelFormat {
                num_channels: 4,
                data_type: JxlDataType::Uint16,
                endianness: JxlEndianness::NativeEndian,
                align: 0,
            },
            img.to_rgba16()
                .as_raw()
                .iter()
                .flat_map(|&pixel| pixel.to_ne_bytes())
                .collect(),
        ),
        _ => {
            return Err(JxlError::UnsupportedOperation(
                "Unsupported image format".to_string(),
            ))
        }
    };

    // 이미지 프레임 추가
    encoder.add_image_frame(frame_settings, &pixel_format, &pixels)?;

    // 입력 종료
    encoder.close_input()?;

    // 출력 데이터 처리
    let mut compressed = Vec::new();

    let mut buffer = vec![0u8; INITIAL_BUFFER_SIZE]; // INITIAL_BUFFER_SIZE는 적절한 초기 크기

    loop {
        let mut next_out = buffer.as_mut_ptr();
        let mut avail_out = buffer.len();

        let status = encoder.process_output(&mut next_out, &mut avail_out)?;
        let bytes_written = buffer.len() - avail_out;
        compressed.extend_from_slice(&buffer[..bytes_written]);

        if status == JxlEncoderStatus::Success {
            break;
        } else if status == JxlEncoderStatus::NeedMoreOutput {
            // 버퍼 크기를 늘려야 할 경우
            // 필요한 경우 버퍼 크기를 늘리고 계속 진행
            buffer.resize(buffer.len() * 2, 0);
        } else {
            return Err(JxlError::EncodingFailed(status));
        }
    }

    Ok(compressed)
}
