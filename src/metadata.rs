use crate::{JxlBool, JxlColorEncoding};

/// Image orientation metadata. Values 1..8 match the EXIF definitions.
/// The name indicates the operation to perform to transform from the encoded image to the display image.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlOrientation {
    ///JXL_ORIENT_IDENTITY
    Identity = 1,
    ///JXL_ORIENT_FLIP_HORIZONTAL
    FlipHorizontal = 2,
    ///JXL_ORIENT_ROTATE_180
    Rotate180 = 3,
    ///JXL_ORIENT_FLIP_VERTICAL
    FlipVertical = 4,
    ///JXL_ORIENT_TRANSPOSE
    Transpose = 5,
    ///JXL_ORIENT_ROTATE_90_CW
    Rotate90CW = 6,
    ///JXL_ORIENT_TRANSVERSE
    AntiTranspose = 7,
    ///JXL_ORIENT_ROTATE_90_CCW
    Rotate90CCW = 8,
}

impl Default for JxlOrientation {
    fn default() -> Self {
        JxlOrientation::Identity
    }
}

/// Given type of an extra channel.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlExtraChannelType {
    ///JXL_CHANNEL_ALPHA
    Alpha,
    ///JXL_CHANNEL_DEPTH
    Depth,
    ///JXL_CHANNEL_SPOT_COLOR
    SpotColor,
    ///JXL_CHANNEL_SELECTION_MASK
    SelectionMask,
    ///JXL_CHANNEL_BLACK
    Black,
    ///JXL_CHANNEL_CFA
    Cfa,
    ///JXL_CHANNEL_THERMAL
    Thermal,
    ///JXL_CHANNEL_RESERVED0
    Reserved0,
    ///JXL_CHANNEL_RESERVED1
    Reserved1,
    ///JXL_CHANNEL_RESERVED2
    Reserved2,
    ///JXL_CHANNEL_RESERVED3
    Reserved3,
    ///JXL_CHANNEL_RESERVED4
    Reserved4,
    ///JXL_CHANNEL_RESERVED5
    Reserved5,
    ///JXL_CHANNEL_RESERVED6
    Reserved6,
    ///JXL_CHANNEL_RESERVED7
    Reserved7,
    ///JXL_CHANNEL_UNKNOWN
    Unknown,
    ///JXL_CHANNEL_OPTIONAL
    Optional,
}

impl Default for JxlExtraChannelType {
    fn default() -> Self {
        JxlExtraChannelType::Alpha
    }
}

/// Frame blend modes. When decoding, if coalescing is enabled (default), this can be ignored.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlBlendMode {
    ///JXL_BLEND_REPLACE
    Replace = 0,
    ///JXL_BLEND_ADD
    Add = 1,
    ///JXL_BLEND_BLEND
    Blend = 2,
    ///JXL_BLEND_MULADD
    Muladd = 3,
    ///JXL_BLEND_MUL
    Mul = 4,
}

impl Default for JxlBlendMode {
    fn default() -> Self {
        JxlBlendMode::Replace
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents the preview header information.
pub struct JxlPreviewHeader {
    ///Preview width in pixels
    pub xsize: u32,
    ///Preview height in pixels
    pub ysize: u32,
}
impl Default for JxlPreviewHeader {
    fn default() -> Self {
        JxlPreviewHeader { xsize: 0, ysize: 0 }
    }
}

/// Represents the animation header information.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JxlAnimationHeader {
    pub tps_numerator: u32,
    pub tps_denominator: u32,
    pub num_loops: u32,
    pub have_timecodes: JxlBool,
}
impl Default for JxlAnimationHeader {
    fn default() -> Self {
        JxlAnimationHeader {
            tps_numerator: 0,
            tps_denominator: 0,
            num_loops: 0,
            have_timecodes: JxlBool::False,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JxlBasicInfo {
    pub have_container: JxlBool,
    pub xsize: u32,
    pub ysize: u32,
    pub bits_per_sample: u32,
    pub exponent_bits_per_sample: u32,
    pub intensity_target: f32,
    pub min_nits: f32,
    pub relative_to_max_display: JxlBool,
    pub linear_below: f32,
    pub uses_original_profile: JxlBool,
    pub have_preview: JxlBool,
    pub have_animation: JxlBool,
    pub orientation: JxlOrientation,
    pub num_color_channels: u32,
    pub num_extra_channels: u32,
    pub alpha_bits: u32,
    pub alpha_exponent_bits: u32,
    pub alpha_premultiplied: JxlBool,
    pub preview: JxlPreviewHeader,
    pub animation: JxlAnimationHeader,
    ///Intrinsic width of the image.
    /// 이미지의 고유 너비.
    pub intrinsic_xsize: u32,
    /// 이미지의 고유 높이.
    pub intrinsic_ysize: u32,
    pub padding: [u8; 100],
}

impl Default for JxlBasicInfo {
    fn default() -> Self {
        JxlBasicInfo {
            have_container: JxlBool::False,
            xsize: 0,
            ysize: 0,
            bits_per_sample: 0,
            exponent_bits_per_sample: 0,
            // 이미지에 존재하는 강도 레벨 의 상한값 (nits). 부호 없는 정수 픽셀 인코딩의 경우, 이는 가장 큰 표현 가능한 값의 밝기입니다.
            // 이미지에는 실제로 이렇게 밝은 픽셀이 반드시 포함되어 있지는 않습니다. 인코더는 히스토그램을 계산하지 않고도 SDR 이미지에 대해 255를 설정할 수 있습니다.
            // 이를 기본값인 0으로 설정하면 libjxl이 색상 인코딩에 따라 합리적인 기본값을 선택합니다.
            intensity_target: 0.0,
            min_nits: 0.0,
            relative_to_max_display: JxlBool::False,
            linear_below: 0.0,
            uses_original_profile: JxlBool::False,
            have_preview: JxlBool::False,
            have_animation: JxlBool::False,
            orientation: JxlOrientation::Identity,
            num_color_channels: 0,
            num_extra_channels: 0,
            alpha_bits: 0,
            alpha_exponent_bits: 0,
            alpha_premultiplied: JxlBool::False,
            preview: JxlPreviewHeader::default(),
            animation: JxlAnimationHeader::default(),
            intrinsic_xsize: 0,
            intrinsic_ysize: 0,
            padding: [0; 100],
        }
    }
}

/// Represents extra channel information.
#[repr(C)]
#[derive(Debug, Default)]
pub struct JxlExtraChannelInfo {
    pub type_: JxlExtraChannelType,
    pub bits_per_sample: u32,
    pub exponent_bits_per_sample: u32,
    pub dim_shift: u32,
    pub name_length: u32,
    pub alpha_premultiplied: JxlBool,
    pub spot_color: [f32; 4],
    pub cfa_channel: u32,
}

#[repr(C)]
pub struct JxlHeaderExtensions {
    pub extensions: u64,
}

/// Represents blend information for a frame or extra channel.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct JxlBlendInfo {
    pub blendmode: JxlBlendMode,
    pub source: u32,
    pub alpha: u32,
    pub clamp: JxlBool,
}

/// Represents layer information in a frame.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct JxlLayerInfo {
    pub have_crop: JxlBool,
    pub crop_x0: i32,
    pub crop_y0: i32,
    pub xsize: u32,
    pub ysize: u32,
    pub blend_info: JxlBlendInfo,
    pub save_as_reference: u32,
}

/// Represents a frame header in JPEG XL.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct JxlFrameHeader {
    pub duration: u32,
    pub timecode: u32,
    pub name_length: u32,
    pub is_last: JxlBool,
    pub layer_info: JxlLayerInfo,
}

///This structure is used to serialize gain map data to and from an input buffer.
/// It holds pointers to sections within the buffer, and different parts of the gain map data such as metadata, ICC profile data, and the gain map itself.
/// The pointers in this structure do not take ownership of the memory they point to.
/// Instead, they reference specific locations within the provided buffer.
/// It is the caller’s responsibility to ensure that the buffer remains valid and is not deallocated as long as these pointers are in use.
/// The structure should be considered as providing a view into the buffer, not as an owner of the data.
#[repr(C)]
#[derive(Debug)]
pub struct JxlGainMapBundle {
    /// Version number of the gain map bundle.
    pub jhgm_version: u8,

    /// Size of the gain map metadata in bytes.
    pub gain_map_metadata_size: u16,

    /// Pointer to the gain map metadata, which is a binary blob following ISO 21496-1.
    /// This pointer references data within the input buffer.
    pub gain_map_metadata: *const u8,

    /// Indicates whether a color encoding is present.
    pub has_color_encoding: JxlBool,

    /// If has_color_encoding is true, this field contains the uncompressed color encoding data.
    pub color_encoding: JxlColorEncoding,

    /// Size of the alternative ICC profile in bytes (compressed size).
    pub alt_icc_size: u32,

    /// Pointer to the compressed ICC profile.
    /// This pointer references data within the input buffer.
    pub alt_icc: *const u8,

    /// Size of the gain map in bytes.
    pub gain_map_size: u32,

    /// Pointer to the gain map data, which is a JPEG XL naked codestream.
    /// This pointer references data within the input buffer.
    pub gain_map: *const u8,
}

impl JxlGainMapBundle {
    /// Creates a new empty gain map bundle with null pointers
    pub fn new() -> Self {
        Self {
            jhgm_version: 0,
            gain_map_metadata_size: 0,
            gain_map_metadata: std::ptr::null(),
            has_color_encoding: JxlBool::False,
            color_encoding: JxlColorEncoding::default(),
            alt_icc_size: 0,
            alt_icc: std::ptr::null(),
            gain_map_size: 0,
            gain_map: std::ptr::null(),
        }
    }

    /// Safely access the gain map metadata as a slice if available
    ///
    /// # Safety
    ///
    /// The caller must ensure that the underlying buffer is still valid
    pub unsafe fn get_metadata(&self) -> Option<&[u8]> {
        if self.gain_map_metadata.is_null() {
            None
        } else {
            Some(std::slice::from_raw_parts(
                self.gain_map_metadata,
                self.gain_map_metadata_size as usize,
            ))
        }
    }

    /// Safely access the alternative ICC profile as a slice if available
    ///
    /// # Safety
    ///
    /// The caller must ensure that the underlying buffer is still valid
    pub unsafe fn get_alt_icc(&self) -> Option<&[u8]> {
        if self.alt_icc.is_null() {
            None
        } else {
            Some(std::slice::from_raw_parts(
                self.alt_icc,
                self.alt_icc_size as usize,
            ))
        }
    }

    /// Safely access the gain map data as a slice if available
    ///
    /// # Safety
    ///
    /// The caller must ensure that the underlying buffer is still valid
    pub unsafe fn get_gain_map(&self) -> Option<&[u8]> {
        if self.gain_map.is_null() {
            None
        } else {
            Some(std::slice::from_raw_parts(
                self.gain_map,
                self.gain_map_size as usize,
            ))
        }
    }
}

impl Default for JxlGainMapBundle {
    fn default() -> Self {
        Self::new()
    }
}
