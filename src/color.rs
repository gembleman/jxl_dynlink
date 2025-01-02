use std::ffi::{c_int, c_void};

/// Color encoding information for JPEG XL images
#[repr(C)]
#[derive(Debug, Default)]
pub struct JxlColorEncoding {
    /// Color space of the image data
    pub color_space: JxlColorSpace,

    /// Built-in white point. If this value is JXL_WHITE_POINT_CUSTOM,
    /// must use the numerical white point values from white_point_xy
    pub white_point: JxlWhitePoint,

    /// Numerical whitepoint values in CIE xy space
    pub white_point_xy: [f64; 2],

    /// Built-in RGB primaries. If this value is JXL_PRIMARIES_CUSTOM,
    /// must use the numerical primaries values below.
    /// This field and the custom values below are unused and must be ignored
    /// if the color space is JXL_COLOR_SPACE_GRAY or JXL_COLOR_SPACE_XYB
    pub primaries: JxlPrimaries,

    /// Numerical red primary values in CIE xy space
    pub primaries_red_xy: [f64; 2],

    /// Numerical green primary values in CIE xy space
    pub primaries_green_xy: [f64; 2],

    /// Numerical blue primary values in CIE xy space
    pub primaries_blue_xy: [f64; 2],

    /// Transfer function if have_gamma is 0
    pub transfer_function: JxlTransferFunction,

    /// Gamma value used when transfer_function is JXL_TRANSFER_FUNCTION_GAMMA
    pub gamma: f64,

    /// Rendering intent defined for the color profile
    pub rendering_intent: JxlRenderingIntent,
}

/// Represents color space for JPEG XL images
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlColorSpace {
    /// Tristimulus RGB color space
    RGB = 0,

    /// Luminance based color space.
    /// The primaries in JxlColorEncoding must be ignored.
    /// This value implies that num_color_channels in JxlBasicInfo is 1,
    /// while any other value implies num_color_channels is 3.
    Gray = 1,

    /// XYB (opsin) color space
    XYB = 2,

    /// None of the other table entries describe the color space appropriately
    Unknown = 3,
}

/// Represents white point.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum JxlWhitePoint {
    /// CIE Standard Illuminant D65: 0.3127, 0.3290
    D65 = 1,

    /// White point must be read from the JxlColorEncoding white_point field,
    /// or as ICC profile. This enum value is not an exact match of the
    /// corresponding CICP value.
    Custom = 2,

    /// CIE Standard Illuminant E (equal-energy): 1/3, 1/3
    E = 10,

    /// DCI-P3 from SMPTE RP 431-2: 0.314, 0.351
    DCI = 11,
}

/// Color primaries for JPEG XL images
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlPrimaries {
    /**The CIE xy values of the red, green and blue primaries are: 0.639998686,
    0.330010138; 0.300003784, 0.600003357; 0.150002046, 0.059997204*/
    SRGB = 1,

    /// Primaries must be read from the JxlColorEncoding primaries_red_xy,
    /// primaries_green_xy and primaries_blue_xy fields, or as ICC profile.
    /// This enum value is not an exact match of the corresponding CICP value.
    Custom = 2,

    /// As specified in Rec. ITU-R BT.2100-1
    BT2100 = 9,

    /// As specified in SMPTE RP 431-2 (DCI-P3)
    P3 = 11,
}

/// Built-in transfer functions for color encoding.
/// Enum values match a subset of CICP (Rec. ITU-T H.273 | ISO/IEC 23091-2:2019(E))
/// unless specified otherwise.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum JxlTransferFunction {
    /// As specified in ITU-R BT.709-6
    /// JXL_TRANSFER_FUNCTION_709
    BT709 = 1,

    /// None of the other table entries describe the transfer function.
    /// JXL_TRANSFER_FUNCTION_UNKNOWN
    Unknown = 2,

    /// The gamma exponent is 1
    /// JXL_TRANSFER_FUNCTION_LINEAR
    Linear = 8,

    /// As specified in IEC 61966-2-1 sRGB
    /// JXL_TRANSFER_FUNCTION_SRGB
    SRGB = 13,

    /// As specified in SMPTE ST 2084
    /// JXL_TRANSFER_FUNCTION_PQ
    PQ = 16,

    /// As specified in SMPTE ST 428-1
    /// JXL_TRANSFER_FUNCTION_DCI
    DCI = 17,

    /// As specified in Rec. ITU-R BT.2100-1 (HLG)
    /// JXL_TRANSFER_FUNCTION_HLG
    HLG = 18,

    /// Transfer function follows power law given by the gamma value in JxlColorEncoding.
    /// Not a CICP value.
    /// JXL_TRANSFER_FUNCTION_GAMMA
    Gamma = 65535,
}

/// Represents rendering intent.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum JxlRenderingIntent {
    ///JXL_RENDERING_INTENT_PERCEPTUAL
    Perceptual = 0,
    ///JXL_RENDERING_INTENT_RELATIVE
    Relative,
    ///JXL_RENDERING_INTENT_SATURATION
    Saturation,
    ///JXL_RENDERING_INTENT_ABSOLUTE
    Absolute,
}

// Default implementations for enums...

impl Default for JxlColorSpace {
    fn default() -> Self {
        JxlColorSpace::RGB
    }
}

impl Default for JxlWhitePoint {
    fn default() -> Self {
        JxlWhitePoint::D65
    }
}

impl Default for JxlPrimaries {
    fn default() -> Self {
        JxlPrimaries::SRGB
    }
}

impl Default for JxlTransferFunction {
    fn default() -> Self {
        JxlTransferFunction::SRGB
    }
}

impl Default for JxlRenderingIntent {
    fn default() -> Self {
        JxlRenderingIntent::Perceptual
    }
}

/// Represents a color management system interface.
#[repr(C)]
pub struct JxlCmsInterface {
    pub get_color_profile_size: Option<unsafe extern "C" fn(*mut c_void, *mut usize) -> c_int>,
    pub get_color_profile: Option<unsafe extern "C" fn(*mut c_void, *mut u8, usize) -> c_int>,
    pub set_color_profile_size: Option<unsafe extern "C" fn(*mut c_void, usize) -> c_int>,
    pub set_color_profile: Option<unsafe extern "C" fn(*mut c_void, *const u8, usize) -> c_int>,
    // Add other CMS-related function pointers as needed
}

// Implement Default for JxlCmsInterface
impl Default for JxlCmsInterface {
    fn default() -> Self {
        JxlCmsInterface {
            get_color_profile_size: None,
            get_color_profile: None,
            set_color_profile_size: None,
            set_color_profile: None,
        }
    }
}
