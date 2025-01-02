use crate::JxlError;

/// Represents the signature check result.
#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum JxlSignature {
    NotEnoughBytes,
    Invalid,
    Codestream,
    Container,
}

/// JxlDecoderProcessInput의 반환 값을 나타냅니다.
/// JXL_DEC_BASIC_INFO 이후의 값들은 선택적인 정보 이벤트로,
/// JxlDecoderSubscribeEvents로 등록된 경우에만 반환됩니다.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlDecoderStatus {
    /// 함수 호출이 성공적으로 완료되었거나 디코딩이 끝나고 더 이상 할 일이 없음을 나타냅니다.
    Success = 0,
    Error = 1,
    NeedMoreInput = 2,
    NeedPreviewOutBuffer = 3,
    /// 디코더가 전체 해상도 이미지를 저장할 출력 버퍼를 요청합니다. -  JXL_DEC_NEED_IMAGE_OUT_BUFFER
    NeedImageOutBuffer = 5,
    JpegNeedMoreOutput = 6,
    BoxNeedMoreOutput = 7,
    BasicInfo = 0x40,
    ColorEncoding = 0x100,
    PreviewImage = 0x200,
    /// 프레임의 시작을 나타냅니다. - JXL_DEC_FRAME
    Frame = 0x400,
    /// JXL_DEC_FULL_IMAGE
    FullImage = 0x1000,
    /// JXL_DEC_JPEG_RECONSTRUCTION
    JpegReconstruction = 0x2000,
    /// JXL_DEC_BOX
    Box = 0x4000,
    /// JXL_DEC_FRAME_PROGRESSION
    FrameProgression = 0x8000,
    /// JXL_DEC_BOX_COMPLETE
    BoxComplete = 0x10000,
}
impl JxlDecoderStatus {
    pub fn bits(&self) -> i32 {
        *self as i32
    }

    pub fn from_bits(bits: i32) -> std::result::Result<Self, JxlError> {
        match bits {
            0 => Ok(JxlDecoderStatus::Success),
            1 => Ok(JxlDecoderStatus::Error),
            2 => Ok(JxlDecoderStatus::NeedMoreInput),
            3 => Ok(JxlDecoderStatus::NeedPreviewOutBuffer),
            5 => Ok(JxlDecoderStatus::NeedImageOutBuffer),
            6 => Ok(JxlDecoderStatus::JpegNeedMoreOutput),
            7 => Ok(JxlDecoderStatus::BoxNeedMoreOutput),
            0x40 => Ok(JxlDecoderStatus::BasicInfo),
            0x100 => Ok(JxlDecoderStatus::ColorEncoding),
            0x200 => Ok(JxlDecoderStatus::PreviewImage),
            0x400 => Ok(JxlDecoderStatus::Frame),
            0x1000 => Ok(JxlDecoderStatus::FullImage),
            0x2000 => Ok(JxlDecoderStatus::JpegReconstruction),
            0x4000 => Ok(JxlDecoderStatus::Box),
            0x8000 => Ok(JxlDecoderStatus::FrameProgression),
            0x10000 => Ok(JxlDecoderStatus::BoxComplete),
            _ => Err(JxlError::InvalidDecoderStatus(bits)),
        }
    }
}

impl std::ops::BitOr for JxlDecoderStatus {
    type Output = i32;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.bits() | rhs.bits()
    }
}
/// Represents progressive detail levels.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlProgressiveDetail {
    KFrames,
    KDc,
    KLastPasses,
    KPasses,
    KDcprogressive,
    KDcgroups,
    KGroups,
}

/// Represents the color profile target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlColorProfileTarget {
    /// JXL_COLOR_PROFILE_TARGET_ORIGINAL
    Original,
    /// JXL_COLOR_PROFILE_TARGET_DATA
    Data,
}
