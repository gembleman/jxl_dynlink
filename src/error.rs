use crate::encoder_enum::JxlEncoderError;
use crate::JxlEncoderStatus;

/// Represents possible errors that can occur during JPEG XL decoding and encoding.
#[derive(Debug)]
pub enum JxlError {
    // 기존 에러들...
    LibraryLoadFailed(libloading::Error),
    SymbolLoadFailed(libloading::Error),
    EncoderCreationFailed,
    NullPointer,
    InvalidInput(String),
    EncodingFailed(JxlEncoderStatus),
    NotEnoughOutput,
    SetParallelRunnerFailed,
    ColorProfileError,
    FrameError(String),
    ExtraChannelError(String),
    BoxError(String),
    UnsupportedOperation(String),
    OutputBufferError,
    PixelFormatError,
    ProgressiveError,
    JpegReconstructionError,
    BoxBufferError,
    ExtraChannelBufferError,
    PreviewBufferError,
    FrameIndexBoxError,
    EncoderFrameSettingsFailed,
    DecoderCreationFailed,
    DecodingFailed,
    NotEnoughInput,

    // ICC 프로필 관련 에러들
    ICCProfileEncodeFailed(String),
    ICCProfileDecodeFailed(String),
    ICCProfileNullPointer,
    ICCProfileInvalidSize,
    ICCProfileCompressionFailed,
    ICCProfileDecompressionFailed,
    ICCProfileMemoryError,

    // GainMap 관련 에러들
    GainMapBundleNullPointer,
    GainMapBundleSizeFailed,
    GainMapBundleWriteFailed(String),
    GainMapBundleReadFailed(String),
    GainMapBundleInvalidSize,
    GainMapBundleInvalidData,
    GainMapBundleMemoryError,

    InvalidDecoderStatus(i32),
}

impl JxlError {
    pub fn from_encoder_status(
        status: JxlEncoderStatus,
        encoder_error: Option<JxlEncoderError>,
    ) -> JxlError {
        match status {
            JxlEncoderStatus::Success => panic!("Success is not an error"),
            JxlEncoderStatus::Error => match encoder_error.unwrap_or(JxlEncoderError::Generic) {
                JxlEncoderError::Ok => JxlError::EncodingFailed(status),
                JxlEncoderError::Generic => JxlError::EncodingFailed(status),
                JxlEncoderError::Oom => JxlError::EncodingFailed(status),
                JxlEncoderError::Jbrd => JxlError::EncodingFailed(status),
                JxlEncoderError::BadInput => JxlError::InvalidInput("Bad input".to_string()),
                JxlEncoderError::NotSupported => {
                    JxlError::UnsupportedOperation("Operation not supported".to_string())
                }
                JxlEncoderError::ApiUsage => {
                    JxlError::InvalidInput("Incorrect API usage".to_string())
                }
            },
            JxlEncoderStatus::NeedMoreOutput => JxlError::NotEnoughOutput,
        }
    }
}
