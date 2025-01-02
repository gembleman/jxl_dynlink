#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlEncoderStatus {
    Success = 0,
    Error = 1,
    NeedMoreOutput = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlEncoderError {
    Ok = 0,
    Generic = 1,
    Oom = 2,
    Jbrd = 3,
    BadInput = 4,
    NotSupported = 0x80,
    ApiUsage = 0x81,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlEncoderFrameSettingId {
    Effort = 0,
    DecodingSpeed = 1,
    Resampling = 2,
    ExtraChannelResampling = 3,
    AlreadyDownsampled = 4,
    PhotonNoise = 5,
    Noise = 6,
    Dots = 7,
    Patches = 8,
    Epf = 9,
    Gaborish = 10,
    Modular = 11,
    KeepInvisible = 12,
    GroupOrder = 13,
    GroupOrderCenterX = 14,
    GroupOrderCenterY = 15,
    Responsive = 16,
    ProgressiveAc = 17,
    QprogressiveAc = 18,
    ProgressiveDc = 19,
    ChannelColorsGlobalPercent = 20,
    ChannelColorsGroupPercent = 21,
    PaletteColors = 22,
    LossyPalette = 23,
    ColorTransform = 24,
    ModularColorSpace = 25,
    ModularGroupSize = 26,
    ModularPredictor = 27,
    ModularMaTreeLearningPercent = 28,
    ModularNbPrevChannels = 29,
    JpegReconCfl = 30,
    IndexBox = 31,
    BrotliEffort = 32,
    JpegCompressBoxes = 33,
    Buffering = 34,
    JpegKeepExif = 35,
    JpegKeepXmp = 36,
    JpegKeepJumbf = 37,
    UseFullImageHeuristics = 38,
    DisablePerceptualHeuristics = 39,
    FillEnum = 65535,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JxlEncoderStatsKey {
    HeaderBits,
    TocBits,
    DictionaryBits,
    SplinesBits,
    NoiseBits,
    QuantBits,
    ModularTreeBits,
    ModularGlobalBits,
    DcBits,
    ModularDcGroupBits,
    ControlFieldsBits,
    CoefOrderBits,
    AcHistogramBits,
    AcBits,
    ModularAcGroupBits,
    NumSmallBlocks,
    NumDct4x8Blocks,
    NumAfvBlocks,
    NumDct8Blocks,
    NumDct8x32Blocks,
    NumDct16Blocks,
    NumDct16x32Blocks,
    NumDct32Blocks,
    NumDct32x64Blocks,
    NumDct64Blocks,
    NumButteraugliIters,
    NumStats,
}
