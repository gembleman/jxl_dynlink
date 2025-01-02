use crate::decoder_enum::*;
use crate::decoder_struct::*;
use crate::JxlBasicInfo;
use crate::JxlBitDepth;
use crate::JxlBlendInfo;
use crate::JxlCmsInterface;
use crate::JxlColorEncoding;
use crate::JxlError;
use crate::JxlExtraChannelInfo;
use crate::JxlFrameHeader;
use crate::JxlPixelFormat;

use libloading::{Library, Symbol};
use std::ffi::{c_char, c_float, c_int, c_void};
use std::ptr;

/// Represents a JPEG XL decoder.
pub struct JxlDecoder {
    lib: Library,
    pub dec: *mut c_void,
}

impl JxlDecoder {
    /// Gets the version of the JPEG XL decoder. - JxlDecoderVersion()
    pub fn version(&self) -> Result<(u32, u32, u32), JxlError> {
        let get_version: Symbol<unsafe extern "C" fn() -> u32> =
            unsafe { self.lib.get(b"JxlDecoderVersion") }.map_err(JxlError::SymbolLoadFailed)?;

        let version = unsafe { get_version() };
        let major = version / 1_000_000;
        let minor = (version % 1_000_000) / 1_000;
        let patch = version % 1_000;

        Ok((major, minor, patch))
    }

    /// Checks the signature of the input data. - JxlSignatureCheck()
    /// Checks if the passed buffer contains a valid JPEG XL signature. The passed buf of size size doesn’t need to be a full image, only the beginning of the file.
    /// 전달된 버퍼에 유효한 JPEG XL 서명이 포함되어 있는지 확인합니다. 전달된 buf의 크기 size는 전체 이미지일 필요가 없으며, 파일의 시작 부분만 있으면 됩니다.
    pub fn check_signature(&self, data: &[u8]) -> Result<JxlSignature, JxlError> {
        let signature_check: Symbol<unsafe extern "C" fn(*const u8, usize) -> c_int> =
            unsafe { self.lib.get(b"JxlSignatureCheck") }.map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { signature_check(data.as_ptr(), data.len()) };
        match result {
            0 => Ok(JxlSignature::NotEnoughBytes),
            1 => Ok(JxlSignature::Invalid),
            2 => Ok(JxlSignature::Codestream),
            3 => Ok(JxlSignature::Container),
            _ => Err(JxlError::InvalidInput(result.to_string())),
        }
    }

    /// Creates a new JPEG XL decoder. - JxlDecoderCreate()
    pub fn new(dll_path: &str) -> Result<Self, JxlError> {
        let lib = unsafe { Library::new(dll_path) }.map_err(JxlError::LibraryLoadFailed)?;

        let create: Symbol<unsafe extern "C" fn(*const c_void) -> *mut c_void> =
            unsafe { lib.get(b"JxlDecoderCreate") }.map_err(JxlError::SymbolLoadFailed)?;

        let dec = unsafe { create(ptr::null()) };
        if dec.is_null() {
            return Err(JxlError::DecoderCreationFailed);
        }

        Ok(JxlDecoder { lib, dec })
    }

    /// Resets the decoder. - JxlDecoderReset()
    pub fn reset(&self) {
        let reset: Symbol<unsafe extern "C" fn(*mut c_void)> =
            unsafe { self.lib.get(b"JxlDecoderReset").unwrap() };

        unsafe { reset(self.dec) };
    }

    /// Rewinds the decoder to the beginning of the input. - JxlDecoderRewind()
    pub fn rewind(&self) {
        let rewind: Symbol<unsafe extern "C" fn(*mut c_void)> =
            unsafe { self.lib.get(b"JxlDecoderRewind").unwrap() };

        unsafe { rewind(self.dec) };
    }

    /// Skips a specified number of frames. - JxlDecoderSkipFrames()
    pub fn skip_frames(&self, amount: usize) {
        let skip_frames: Symbol<unsafe extern "C" fn(*mut c_void, usize)> =
            unsafe { self.lib.get(b"JxlDecoderSkipFrames").unwrap() };

        unsafe { skip_frames(self.dec, amount) };
    }

    /// Skips decoding the current frame. - JxlDecoderSkipCurrentFrame()
    pub fn skip_current_frame(&self) -> Result<(), JxlError> {
        let skip_current_frame: Symbol<unsafe extern "C" fn(*mut c_void) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderSkipCurrentFrame") }
                .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { skip_current_frame(self.dec) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::FrameError(result.to_string()))
        }
    }

    /// Sets the parallel runner for multithreading. - JxlDecoderSetParallelRunner()
    pub fn set_parallel_runner(
        &self,
        parallel_runner: Option<JxlParallelRunner>,
        parallel_runner_opaque: *mut c_void,
    ) -> Result<(), JxlError> {
        let set_parallel_runner: Symbol<
            unsafe extern "C" fn(*mut c_void, Option<JxlParallelRunner>, *mut c_void) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderSetParallelRunner") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let result =
            unsafe { set_parallel_runner(self.dec, parallel_runner, parallel_runner_opaque) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::SetParallelRunnerFailed)
        }
    }

    /// Gets the size hint for basic info. - JxlDecoderSizeHintBasicInfo()
    pub fn size_hint_basic_info(&self) -> usize {
        let size_hint_basic_info: Symbol<unsafe extern "C" fn(*const c_void) -> usize> =
            unsafe { self.lib.get(b"JxlDecoderSizeHintBasicInfo").unwrap() };

        unsafe { size_hint_basic_info(self.dec) }
    }

    /// Subscribes to decoder events. - JxlDecoderSubscribeEvents()
    pub fn subscribe_events(&self, events: i32) -> Result<(), JxlError> {
        let subscribe_events: Symbol<unsafe extern "C" fn(*mut c_void, c_int) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderSubscribeEvents") }
                .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { subscribe_events(self.dec, events as i32) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::InvalidInput(result.to_string()))
        }
    }

    /// Sets whether to keep the original image orientation. - JxlDecoderSetKeepOrientation()
    pub fn set_keep_orientation(&self, keep: bool) -> Result<(), JxlError> {
        let set_keep_orientation: Symbol<unsafe extern "C" fn(*mut c_void, c_int) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderSetKeepOrientation") }
                .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { set_keep_orientation(self.dec, keep as c_int) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::InvalidInput(result.to_string()))
        }
    }

    /// Sets the unpremultiply alpha option. - JxlDecoderSetUnpremultiplyAlpha()
    pub fn set_unpremultiply_alpha(&self, unpremultiply_alpha: bool) -> Result<(), JxlError> {
        let set_unpremultiply_alpha: Symbol<unsafe extern "C" fn(*mut c_void, c_int) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderSetUnpremultiplyAlpha") }
                .map_err(JxlError::SymbolLoadFailed)?;

        let result =
            unsafe { set_unpremultiply_alpha(self.dec, if unpremultiply_alpha { 1 } else { 0 }) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::InvalidInput(result.to_string()))
        }
    }

    /// Sets the render spotcolors option. - JxlDecoderSetRenderSpotcolors()
    pub fn set_render_spotcolors(&self, render_spotcolors: bool) -> Result<(), JxlError> {
        let set_render_spotcolors: Symbol<unsafe extern "C" fn(*mut c_void, c_int) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderSetRenderSpotcolors") }
                .map_err(JxlError::SymbolLoadFailed)?;

        let result =
            unsafe { set_render_spotcolors(self.dec, if render_spotcolors { 1 } else { 0 }) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::InvalidInput(result.to_string()))
        }
    }

    /// Sets the coalescing option for animated images. - JxlDecoderSetCoalescing()
    pub fn set_coalescing(&self, coalescing: bool) -> Result<(), JxlError> {
        let set_coalescing: Symbol<unsafe extern "C" fn(*mut c_void, c_int) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderSetCoalescing") }
                .map_err(JxlError::SymbolLoadFailed)?;

        let result_bits = unsafe { set_coalescing(self.dec, if coalescing { 1 } else { 0 }) };
        let result = JxlDecoderStatus::from_bits(result_bits)?;
        if result == JxlDecoderStatus::Success {
            Ok(())
        } else {
            Err(JxlError::InvalidInput(result_bits.to_string()))
        }
    }

    /// Processes the input data. - JxlDecoderProcessInput()
    pub fn process_input(&self) -> Result<JxlDecoderStatus, JxlError> {
        let process_input: Symbol<unsafe extern "C" fn(*mut c_void) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderProcessInput") }
                .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { process_input(self.dec) };
        match result {
            0 => Ok(JxlDecoderStatus::Success),
            1 => Err(JxlError::DecodingFailed),
            2 => Ok(JxlDecoderStatus::NeedMoreInput),
            64 => Ok(JxlDecoderStatus::BasicInfo),
            128 => Ok(JxlDecoderStatus::ColorEncoding),
            256 => Ok(JxlDecoderStatus::PreviewImage),
            512 => Ok(JxlDecoderStatus::Frame),
            1024 => Ok(JxlDecoderStatus::FullImage),
            _ => Err(JxlError::InvalidInput(result.to_string())),
        }
    }

    /// Sets the input data for the decoder. - JxlDecoderSetInput()
    pub fn set_input(&self, data: &[u8]) -> Result<(), JxlError> {
        let set_input: Symbol<unsafe extern "C" fn(*mut c_void, *const u8, usize) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderSetInput") }.map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { set_input(self.dec, data.as_ptr(), data.len()) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::InvalidInput(result.to_string()))
        }
    }

    /// Releases input which was provided with JxlDecoderSetInput. - JxlDecoderReleaseInput()
    pub fn release_input(&self) -> usize {
        let release_input: Symbol<unsafe extern "C" fn(*mut c_void) -> usize> =
            unsafe { self.lib.get(b"JxlDecoderReleaseInput").unwrap() };

        unsafe { release_input(self.dec) }
    }

    /// Closes the input, indicating no more input will be set. - JxlDecoderCloseInput()
    pub fn close_input(&self) {
        let close_input: Symbol<unsafe extern "C" fn(*mut c_void)> =
            unsafe { self.lib.get(b"JxlDecoderCloseInput").unwrap() };

        unsafe { close_input(self.dec) };
    }

    /// Gets the basic information about the image. - JxlDecoderGetBasicInfo()
    pub fn get_basic_info(&self) -> Result<JxlBasicInfo, JxlError> {
        let get_basic_info: Symbol<
            unsafe extern "C" fn(*const c_void, *mut JxlBasicInfo) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderGetBasicInfo") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let mut info = JxlBasicInfo::default();
        let result = unsafe { get_basic_info(self.dec, &mut info) };
        if result == 0 {
            Ok(info)
        } else {
            Err(JxlError::InvalidInput(result.to_string()))
        }
    }

    /// Gets the extra channel information. - JxlDecoderGetExtraChannelInfo()
    pub fn get_extra_channel_info(&self, index: usize) -> Result<JxlExtraChannelInfo, JxlError> {
        let get_extra_channel_info: Symbol<
            unsafe extern "C" fn(*const c_void, usize, *mut JxlExtraChannelInfo) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderGetExtraChannelInfo") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let mut extra_channel_info = JxlExtraChannelInfo::default();
        let result = unsafe { get_extra_channel_info(self.dec, index, &mut extra_channel_info) };
        if result == 0 {
            Ok(extra_channel_info)
        } else {
            Err(JxlError::ExtraChannelError(result.to_string()))
        }
    }

    /// Gets the name of an extra channel. - JxlDecoderGetExtraChannelName()
    pub fn get_extra_channel_name(&self, index: usize) -> Result<String, JxlError> {
        let get_extra_channel_name: Symbol<
            unsafe extern "C" fn(*const c_void, usize, *mut c_char, usize) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderGetExtraChannelName") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let size: usize = 0;
        let result = unsafe { get_extra_channel_name(self.dec, index, ptr::null_mut(), size) };
        if result != 0 {
            return Err(JxlError::ExtraChannelError(result.to_string()));
        }

        let mut buffer = vec![0u8; size];
        let result = unsafe {
            get_extra_channel_name(self.dec, index, buffer.as_mut_ptr() as *mut c_char, size)
        };
        if result == 0 {
            Ok(String::from_utf8_lossy(&buffer[..size - 1]).into_owned())
        } else {
            Err(JxlError::ExtraChannelError(result.to_string()))
        }
    }

    /// Gets the color encoding information. - JxlDecoderGetColorAsEncodedProfile()
    /// 가능한 경우 JPEG XL 인코딩된 구조화된 데이터로 색상 프로파일을 출력합니다. 이는 ICC 프로파일의 대안으로, ICC 프로파일은 더 제한된 수의 색 공간을 표현할 수 있지만 열거형 값을 통해 정확하게 표현합니다.
    /// 대안으로 JxlDecoderGetColorAsICCProfile을 사용하는 것도 종종 가능합니다. 다음과 같은 시나리오가 가능합니다:
    /// JPEG XL 이미지에 ICC 프로파일이 첨부된 경우, 인코딩된 구조화된 데이터를 사용할 수 없으며 이 함수는 오류 상태를 반환합니다. 대신 JxlDecoderGetColorAsICCProfile을 호출해야 합니다.
    /// JPEG XL 이미지에 인코딩된 구조화된 색상 프로파일이 있고, RGB 또는 그레이스케일 색 공간을 나타내는 경우. 이 함수는 해당 프로파일을 반환합니다. 원한다면 여전히 JxlDecoderGetColorAsICCProfile을 대안으로 사용할 수 있지만, 표현된 RGB 색 공간에 따라 ICC 프로파일이 근사치일 수 있습니다. 또한 ICC 프로파일이 임의의 공간을 나타낼 수 있기 때문에, ICC 프로파일에서 정확히 어떤 명명된 색 공간을 나타내는지 추론하는 것이 항상 가능한 것은 아닙니다. PQ와 HLG를 사용하는 HDR 색 공간도 잠재적으로 문제가 될 수 있습니다. ICC 프로파일이 PQ와 HLG의 전달 함수를 근사할 수 있지만(HLG의 경우 주어진 시스템 감마에 대해서만, 그리고 감마가 '1'과 다를 경우 3D LUT가 필요함), ICCv4.4 이전에는 이것이 그들이 나타내는 색 공간임을 의미론적으로 신호할 수 없습니다. 따라서 일반적으로 HDR 색 공간을 나타내는 것으로 해석되지 않습니다. 이는 특히 PQ에 해로운데, 최대 신호값이 10000 cd/m^2 대신 SDR 화이트를 나타내는 것처럼 해석되어 이미지가 2단계의 크기(5-7 EV)만큼 너무 어둡게 표시됩니다.
    /// JPEG XL 이미지에 인코딩된 구조화된 색상 프로파일이 있고, 알 수 없는 색 공간이나 xyb 색 공간을 나타내는 경우. 이 경우 JxlDecoderGetColorAsICCProfile을 사용할 수 없습니다.
    pub fn get_color_as_encoded_profile(
        &self,
        target: JxlColorProfileTarget,
    ) -> Result<JxlColorEncoding, JxlError> {
        let get_color_as_encoded_profile: Symbol<
            unsafe extern "C" fn(
                *const c_void,
                JxlColorProfileTarget,
                *mut JxlColorEncoding,
            ) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderGetColorAsEncodedProfile") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let mut color_encoding = JxlColorEncoding::default();
        let result = unsafe { get_color_as_encoded_profile(self.dec, target, &mut color_encoding) };
        if result == 0 {
            Ok(color_encoding)
        } else {
            Err(JxlError::ColorProfileError)
        }
    }

    /// Gets the ICC profile size. JxlDecoderGetICCProfileSize()
    ///
    /// # Arguments
    ///
    /// * `target` - Whether to get the original color profile from the metadata
    ///              or the color profile of the decoded pixels.
    /// * `size` - Optional pointer to output the size into. If None, only checks the return status.
    ///
    /// # Returns
    ///
    /// * `Ok(size)` - The size of the ICC profile in bytes
    /// * `Err(JxlError)` - If the profile is not available or there's not enough input
    ///
    pub fn get_icc_profile_size(
        &self,
        target: JxlColorProfileTarget,
        size: Option<&mut usize>,
    ) -> Result<(), JxlError> {
        let get_icc_profile_size: Symbol<
            unsafe extern "C" fn(
                *const c_void,
                JxlColorProfileTarget,
                *mut usize,
            ) -> JxlDecoderStatus,
        > = unsafe { self.lib.get(b"JxlDecoderGetICCProfileSize") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let status = unsafe {
            get_icc_profile_size(
                self.dec,
                target,
                size.map_or(std::ptr::null_mut(), |s| s as *mut usize),
            )
        };

        match status {
            JxlDecoderStatus::Success => Ok(()),
            JxlDecoderStatus::NeedMoreInput => Err(JxlError::NotEnoughInput),
            _ => Err(JxlError::ColorProfileError),
        }
    }

    /// Gets the ICC profile. JxlDecoderGetColorAsICCProfile()
    ///
    /// # Arguments
    ///
    /// * `target` - Whether to get the original color profile from the metadata
    ///              or the color profile of the decoded pixels
    /// * `icc_profile` - Buffer to copy the ICC profile into
    /// * `size` - Size of the icc_profile buffer in bytes
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The profile was successfully returned
    /// * `Err(JxlError)` - If the profile is not available or there's not enough input
    pub fn get_color_as_icc_profile(
        &self,
        target: JxlColorProfileTarget,
        icc_profile: &mut [u8],
        size: usize,
    ) -> Result<(), JxlError> {
        let get_color_as_icc: Symbol<
            unsafe extern "C" fn(
                *const c_void,
                JxlColorProfileTarget,
                *mut u8,
                usize,
            ) -> JxlDecoderStatus,
        > = unsafe { self.lib.get(b"JxlDecoderGetColorAsICCProfile") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let status = unsafe { get_color_as_icc(self.dec, target, icc_profile.as_mut_ptr(), size) };

        match status {
            JxlDecoderStatus::Success => Ok(()),
            JxlDecoderStatus::NeedMoreInput => Err(JxlError::NotEnoughInput),
            _ => Err(JxlError::ColorProfileError),
        }
    }

    /// Sets the preferred color profile for decoding. - JxlDecoderSetPreferredColorProfile()
    pub fn set_preferred_color_profile(
        &self,
        color_encoding: &JxlColorEncoding,
    ) -> Result<(), JxlError> {
        use crate::decoder_enum::JxlDecoderStatus;

        let set_preferred_color_profile: Symbol<
            unsafe extern "C" fn(*mut c_void, *const JxlColorEncoding) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderSetPreferredColorProfile") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { set_preferred_color_profile(self.dec, color_encoding) };
        // 0 is JXL_DEC_SUCCESS

        let result = JxlDecoderStatus::from_bits(result)?;

        if result == JxlDecoderStatus::Success {
            Ok(())
        } else {
            Err(JxlError::ColorProfileError)
        }
    }

    /// Sets the desired intensity target for HDR images. - JxlDecoderSetDesiredIntensityTarget()
    pub fn set_desired_intensity_target(
        &self,
        desired_intensity_target: f32,
    ) -> Result<(), JxlError> {
        let set_desired_intensity_target: Symbol<
            unsafe extern "C" fn(*mut c_void, c_float) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderSetDesiredIntensityTarget") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { set_desired_intensity_target(self.dec, desired_intensity_target) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::InvalidInput(result.to_string()))
        }
    }

    /// Sets the output color profile for decoding. - JxlDecoderSetOutputColorProfile()
    pub fn set_output_color_profile(
        &self,
        color_encoding: Option<&JxlColorEncoding>,
        icc_data: Option<&[u8]>,
    ) -> Result<(), JxlError> {
        let set_output_color_profile: Symbol<
            unsafe extern "C" fn(*mut c_void, *const JxlColorEncoding, *const u8, usize) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderSetOutputColorProfile") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let (color_encoding_ptr, icc_data_ptr, icc_size) = match (color_encoding, icc_data) {
            (Some(ce), None) => (ce as *const JxlColorEncoding, std::ptr::null(), 0),
            (None, Some(icc)) => (std::ptr::null(), icc.as_ptr(), icc.len()),
            _ => {
                return Err(JxlError::InvalidInput(
                    "Both color encoding and ICC data provided".to_string(),
                ))
            }
        };

        let result = unsafe {
            set_output_color_profile(self.dec, color_encoding_ptr, icc_data_ptr, icc_size)
        };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::ColorProfileError)
        }
    }

    /// Sets the color management system (CMS) to use for color conversions. - JxlDecoderSetCms()
    pub fn set_cms(&self, cms: &JxlCmsInterface) -> Result<(), JxlError> {
        let set_cms: Symbol<unsafe extern "C" fn(*mut c_void, *const JxlCmsInterface) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderSetCms") }.map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { set_cms(self.dec, cms) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::ColorProfileError)
        }
    }

    /// Gets the size needed for the preview image output buffer. - JxlDecoderPreviewOutBufferSize()
    pub fn preview_out_buffer_size(&self, format: &JxlPixelFormat) -> Result<usize, JxlError> {
        let preview_out_buffer_size: Symbol<
            unsafe extern "C" fn(*const c_void, *const JxlPixelFormat, *mut usize) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderPreviewOutBufferSize") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let mut size = 0;
        let result = unsafe { preview_out_buffer_size(self.dec, format, &mut size) };
        if result == 0 {
            Ok(size)
        } else {
            Err(JxlError::PreviewBufferError)
        }
    }

    /// Sets the preview image output buffer. - JxlDecoderSetPreviewOutBuffer()
    pub fn set_preview_out_buffer(
        &self,
        format: &JxlPixelFormat,
        buffer: &mut [u8],
    ) -> Result<(), JxlError> {
        let set_preview_out_buffer: Symbol<
            unsafe extern "C" fn(*mut c_void, *const JxlPixelFormat, *mut c_void, usize) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderSetPreviewOutBuffer") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe {
            set_preview_out_buffer(
                self.dec,
                format,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len(),
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::PreviewBufferError)
        }
    }

    /// Gets the frame header for the current frame. - JxlDecoderGetFrameHeader()
    pub fn get_frame_header(&self) -> Result<JxlFrameHeader, JxlError> {
        let get_frame_header: Symbol<
            unsafe extern "C" fn(*const c_void, *mut JxlFrameHeader) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderGetFrameHeader") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let mut frame_header = JxlFrameHeader::default();
        let result = unsafe { get_frame_header(self.dec, &mut frame_header) };
        if result == 0 {
            Ok(frame_header)
        } else {
            Err(JxlError::FrameError(result.to_string()))
        }
    }

    /// Gets the name of the current frame. - JxlDecoderGetFrameName()
    pub fn get_frame_name(&self) -> Result<String, JxlError> {
        let get_frame_name: Symbol<
            unsafe extern "C" fn(*const c_void, *mut c_char, usize) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderGetFrameName") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let mut buffer = vec![0u8; 256]; // Assuming a reasonable max length for frame names
        let result =
            unsafe { get_frame_name(self.dec, buffer.as_mut_ptr() as *mut c_char, buffer.len()) };
        if result == 0 {
            Ok(String::from_utf8_lossy(&buffer)
                .trim_end_matches('\0')
                .to_string())
        } else {
            Err(JxlError::FrameError(result.to_string()))
        }
    }

    /// Gets the blend info for an extra channel in the current frame. - JxlDecoderGetExtraChannelBlendInfo()
    pub fn get_extra_channel_blend_info(&self, index: usize) -> Result<JxlBlendInfo, JxlError> {
        let get_extra_channel_blend_info: Symbol<
            unsafe extern "C" fn(*const c_void, usize, *mut JxlBlendInfo) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderGetExtraChannelBlendInfo") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let mut blend_info = JxlBlendInfo::default();
        let result = unsafe { get_extra_channel_blend_info(self.dec, index, &mut blend_info) };
        if result == 0 {
            Ok(blend_info)
        } else {
            Err(JxlError::ExtraChannelError(result.to_string()))
        }
    }

    /// Gets the output buffer size needed for the decoded image. - JxlDecoderImageOutBufferSize()
    pub fn image_out_buffer_size(&self, format: &JxlPixelFormat) -> Result<usize, JxlError> {
        let image_out_buffer_size: Symbol<
            unsafe extern "C" fn(*const c_void, *const JxlPixelFormat, *mut usize) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderImageOutBufferSize") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let mut size = 0;
        let result = unsafe { image_out_buffer_size(self.dec, format, &mut size) };
        if result == 0 {
            Ok(size)
        } else {
            Err(JxlError::OutputBufferError)
        }
    }

    /// Sets the output buffer for the decoded image. - JxlDecoderSetImageOutBuffer()
    pub fn set_image_out_buffer(
        &self,
        format: &JxlPixelFormat,
        buffer: &mut [u8],
    ) -> Result<(), JxlError> {
        let set_image_out_buffer: Symbol<
            unsafe extern "C" fn(*mut c_void, *const JxlPixelFormat, *mut c_void, usize) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderSetImageOutBuffer") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe {
            set_image_out_buffer(
                self.dec,
                format,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len(),
            )
        };
        if result == 0 {
            Ok(())
        } else {
            println!("Error: {}", result);
            Err(JxlError::OutputBufferError)
        }
    }

    /// Sets a callback for progressive image output. - JxlDecoderSetImageOutCallback()
    pub fn set_image_out_callback<F>(
        &self,
        format: &JxlPixelFormat,
        callback: F,
    ) -> Result<(), JxlError>
    where
        F: FnMut(*mut c_void, usize, usize, usize, *const c_void) + 'static,
    {
        let set_image_out_callback: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                *const JxlPixelFormat,
                unsafe extern "C" fn(*mut c_void, usize, usize, usize, *const c_void),
                *mut c_void,
            ) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderSetImageOutCallback") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let callback_ptr = Box::into_raw(Box::new(callback)) as *mut c_void;

        unsafe extern "C" fn callback_wrapper<F>(
            opaque: *mut c_void,
            x: usize,
            y: usize,
            num_pixels: usize,
            pixels: *const c_void,
        ) where
            F: FnMut(*mut c_void, usize, usize, usize, *const c_void),
        {
            let callback = &mut *(opaque as *mut F);
            callback(opaque, x, y, num_pixels, pixels);
        }

        let result = unsafe {
            set_image_out_callback(self.dec, format, callback_wrapper::<F>, callback_ptr)
        };
        if result == 0 {
            Ok(())
        } else {
            unsafe {
                let _ = Box::from_raw(callback_ptr as *mut F);
            }; // Clean up the box if there's an error
            Err(JxlError::OutputBufferError)
        }
    }

    /// Sets a multi-threaded image output callback. - JxlDecoderSetMultithreadedImageOutCallback()
    pub fn set_multithreaded_image_out_callback<F, G, H>(
        &self,
        format: &JxlPixelFormat,
        init_callback: F,
        run_callback: G,
        destroy_callback: Option<H>,
        _init_opaque: *mut c_void,
    ) -> Result<(), JxlError>
    where
        F: FnMut(*mut c_void, usize, usize) -> *mut c_void + 'static,
        G: FnMut(*mut c_void, usize, usize, usize, usize, *const c_void) + 'static,
        H: FnMut(*mut c_void) + 'static + Copy,
    {
        struct CallbackWrapper<F, G, H> {
            init: F,
            run: G,
            destroy: Option<H>,
        }

        let callbacks = Box::new(CallbackWrapper {
            init: init_callback,
            run: run_callback,
            destroy: destroy_callback,
        });
        let callbacks_ptr = Box::into_raw(callbacks) as *mut c_void;

        unsafe extern "C" fn init_wrapper<F, G, H>(
            init_opaque: *mut c_void,
            num_threads: usize,
            num_pixels_per_thread: usize,
        ) -> *mut c_void
        where
            F: FnMut(*mut c_void, usize, usize) -> *mut c_void,
        {
            let callbacks = &mut *(init_opaque as *mut CallbackWrapper<F, G, H>);
            (callbacks.init)(init_opaque, num_threads, num_pixels_per_thread)
        }

        unsafe extern "C" fn run_wrapper<F, G, H>(
            run_opaque: *mut c_void,
            thread_id: usize,
            x: usize,
            y: usize,
            num_pixels: usize,
            pixels: *const c_void,
        ) where
            G: FnMut(*mut c_void, usize, usize, usize, usize, *const c_void),
        {
            let callbacks = &mut *(run_opaque as *mut CallbackWrapper<F, G, H>);
            (callbacks.run)(run_opaque, thread_id, x, y, num_pixels, pixels);
        }

        unsafe extern "C" fn destroy_wrapper<F, G, H>(run_opaque: *mut c_void)
        where
            H: FnMut(*mut c_void),
        {
            let mut callbacks = Box::from_raw(run_opaque as *mut CallbackWrapper<F, G, H>);
            if let Some(ref mut destroy_callback) = callbacks.destroy {
                destroy_callback(run_opaque);
            }
        }

        let set_multithreaded_image_out_callback: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                *const JxlPixelFormat,
                Option<unsafe extern "C" fn(*mut c_void, usize, usize) -> *mut c_void>,
                Option<unsafe extern "C" fn(*mut c_void, usize, usize, usize, usize, *const c_void)>,
                Option<unsafe extern "C" fn(*mut c_void)>,
                *mut c_void,
            ) -> c_int,
        > = unsafe {
            self.lib
                .get(b"JxlDecoderSetMultithreadedImageOutCallback")
                .map_err(JxlError::SymbolLoadFailed)?
        };

        let result = unsafe {
            set_multithreaded_image_out_callback(
                self.dec,
                format,
                Some(init_wrapper::<F, G, H>),
                Some(run_wrapper::<F, G, H>),
                if destroy_callback.is_some() {
                    Some(destroy_wrapper::<F, G, H>)
                } else {
                    None
                },
                callbacks_ptr,
            )
        };

        if result == 0 {
            Ok(())
        } else {
            unsafe {
                let _ = Box::from_raw(callbacks_ptr as *mut CallbackWrapper<F, G, H>);
            };
            Err(JxlError::OutputBufferError)
        }
    }

    /// Gets the size needed for an extra channel buffer. - JxlDecoderExtraChannelBufferSize()
    pub fn extra_channel_buffer_size(
        &self,
        format: &JxlPixelFormat,
        index: u32,
    ) -> Result<usize, JxlError> {
        let extra_channel_buffer_size: Symbol<
            unsafe extern "C" fn(*const c_void, *const JxlPixelFormat, *mut usize, u32) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderExtraChannelBufferSize") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let mut size = 0;
        let result = unsafe { extra_channel_buffer_size(self.dec, format, &mut size, index) };
        if result == 0 {
            Ok(size)
        } else {
            Err(JxlError::ExtraChannelBufferError)
        }
    }

    /// Sets the buffer for an extra channel. - JxlDecoderSetExtraChannelBuffer()
    pub fn set_extra_channel_buffer(
        &self,
        format: &JxlPixelFormat,
        buffer: &mut [u8],
        index: u32,
    ) -> Result<(), JxlError> {
        let set_extra_channel_buffer: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                *const JxlPixelFormat,
                *mut c_void,
                usize,
                u32,
            ) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderSetExtraChannelBuffer") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe {
            set_extra_channel_buffer(
                self.dec,
                format,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len(),
                index,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::ExtraChannelBufferError)
        }
    }

    /// Sets the buffer for JPEG reconstruction. - JxlDecoderSetJPEGBuffer()
    pub fn set_jpeg_buffer(&self, buffer: &mut [u8]) -> Result<(), JxlError> {
        let set_jpeg_buffer: Symbol<unsafe extern "C" fn(*mut c_void, *mut u8, usize) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderSetJPEGBuffer") }
                .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { set_jpeg_buffer(self.dec, buffer.as_mut_ptr(), buffer.len()) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::JpegReconstructionError)
        }
    }

    /// Releases the JPEG buffer. - JxlDecoderReleaseJPEGBuffer()
    pub fn release_jpeg_buffer(&self) -> usize {
        let release_jpeg_buffer: Symbol<unsafe extern "C" fn(*mut c_void) -> usize> =
            unsafe { self.lib.get(b"JxlDecoderReleaseJPEGBuffer").unwrap() };

        unsafe { release_jpeg_buffer(self.dec) }
    }

    /// Sets the buffer for box output. - JxlDecoderSetBoxBuffer()
    pub fn set_box_buffer(&self, buffer: &mut [u8]) -> Result<(), JxlError> {
        let set_box_buffer: Symbol<unsafe extern "C" fn(*mut c_void, *mut u8, usize) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderSetBoxBuffer") }
                .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { set_box_buffer(self.dec, buffer.as_mut_ptr(), buffer.len()) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::BoxBufferError)
        }
    }

    /// Releases the box buffer. - JxlDecoderReleaseBoxBuffer()
    pub fn release_box_buffer(&self) -> usize {
        let release_box_buffer: Symbol<unsafe extern "C" fn(*mut c_void) -> usize> =
            unsafe { self.lib.get(b"JxlDecoderReleaseBoxBuffer").unwrap() };

        unsafe { release_box_buffer(self.dec) }
    }

    /// Sets whether to decompress boxes. - JxlDecoderSetDecompressBoxes()
    pub fn set_decompress_boxes(&self, decompress: bool) -> Result<(), JxlError> {
        let set_decompress_boxes: Symbol<unsafe extern "C" fn(*mut c_void, c_int) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderSetDecompressBoxes") }
                .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { set_decompress_boxes(self.dec, if decompress { 1 } else { 0 }) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::BoxBufferError)
        }
    }

    /// Gets the type of the current box. - JxlDecoderGetBoxType()
    pub fn get_box_type(&self) -> Result<[u8; 4], JxlError> {
        let get_box_type: Symbol<unsafe extern "C" fn(*const c_void, *mut [u8; 4]) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderGetBoxType") }.map_err(JxlError::SymbolLoadFailed)?;

        let mut box_type = [0u8; 4];
        let result = unsafe { get_box_type(self.dec, &mut box_type) };
        if result == 0 {
            Ok(box_type)
        } else {
            Err(JxlError::BoxError(result.to_string()))
        }
    }

    /// Gets the raw size of the current box. - JxlDecoderGetBoxSizeRaw()
    pub fn get_box_size_raw(&self) -> Result<u64, JxlError> {
        let get_box_size_raw: Symbol<unsafe extern "C" fn(*const c_void, *mut u64) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderGetBoxSizeRaw") }
                .map_err(JxlError::SymbolLoadFailed)?;

        let mut size = 0;
        let result = unsafe { get_box_size_raw(self.dec, &mut size) };
        if result == 0 {
            Ok(size)
        } else {
            Err(JxlError::BoxError(result.to_string()))
        }
    }

    /// Gets the size of the contents of the current box. - JxlDecoderGetBoxSizeContents()
    pub fn get_box_size_contents(&self) -> Result<u64, JxlError> {
        let get_box_size_contents: Symbol<unsafe extern "C" fn(*const c_void, *mut u64) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderGetBoxSizeContents") }
                .map_err(JxlError::SymbolLoadFailed)?;

        let mut size = 0;
        let result = unsafe { get_box_size_contents(self.dec, &mut size) };
        if result == 0 {
            Ok(size)
        } else {
            Err(JxlError::BoxError(result.to_string()))
        }
    }

    /// Sets the level of progressive detail to decode. - JxlDecoderSetProgressiveDetail()
    pub fn set_progressive_detail(&self, detail: JxlProgressiveDetail) -> Result<(), JxlError> {
        let set_progressive_detail: Symbol<
            unsafe extern "C" fn(*mut c_void, JxlProgressiveDetail) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderSetProgressiveDetail") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { set_progressive_detail(self.dec, detail) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::ProgressiveError)
        }
    }

    /// Gets the intended downsampling ratio for the current progressive step. - JxlDecoderGetIntendedDownsamplingRatio()
    pub fn get_intended_downsampling_ratio(&self) -> usize {
        let get_intended_downsampling_ratio: Symbol<unsafe extern "C" fn(*mut c_void) -> usize> = unsafe {
            self.lib
                .get(b"JxlDecoderGetIntendedDownsamplingRatio")
                .unwrap()
        };

        unsafe { get_intended_downsampling_ratio(self.dec) }
    }

    /// Flushes the decoder, returning any partial image data. - JxlDecoderFlushImage()
    pub fn flush_image(&self) -> Result<(), JxlError> {
        let flush_image: Symbol<unsafe extern "C" fn(*mut c_void) -> c_int> =
            unsafe { self.lib.get(b"JxlDecoderFlushImage") }.map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { flush_image(self.dec) };
        if result == 0 {
            Ok(())
        } else {
            Err(JxlError::OutputBufferError)
        }
    }

    /// Sets the desired output bit depth. - JxlDecoderSetImageOutBitDepth()
    /// 출력 버퍼나 콜백의 비트 심도를 설정합니다.
    /// decorder.set_image_out_buffer 또는 decorder.set_image_out_callback 이후에 호출할 수 있습니다.
    /// float 픽셀 데이터 유형의 경우 기본 JXL_BIT_DEPTH_FROM_PIXEL_FORMAT 설정만 지원됩니다.
    pub fn set_image_out_bit_depth(&self, bit_depth: &JxlBitDepth) -> Result<(), JxlError> {
        let set_image_out_bit_depth: Symbol<
            unsafe extern "C" fn(*mut c_void, *const JxlBitDepth) -> c_int,
        > = unsafe { self.lib.get(b"JxlDecoderSetImageOutBitDepth") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let result = unsafe { set_image_out_bit_depth(self.dec, bit_depth) };
        if result == 0 {
            Ok(())
        } else {
            println!("Error: {}", result);
            Err(JxlError::PixelFormatError)
        }
    }
}

impl Drop for JxlDecoder {
    /// Destroys the decoder. - JxlDecoderDestroy()
    fn drop(&mut self) {
        let destroy: Symbol<unsafe extern "C" fn(*mut c_void)> =
            unsafe { self.lib.get(b"JxlDecoderDestroy").unwrap() };
        unsafe { destroy(self.dec) };
    }
}
