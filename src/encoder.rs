use crate::common::*;
use crate::encoder_enum::*;
use crate::encoder_struct::*;
use crate::metadata::*;
use crate::JxlCmsInterface;
use crate::JxlColorEncoding;
use crate::JxlError;
use crate::JxlParallelRunner;

use libloading::{Library, Symbol};

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
use std::ptr;

#[derive(Debug)]
pub struct JxlEncoder {
    pub lib: Library,
    pub enc: *mut c_void,
}

impl JxlEncoder {
    /// Get the version of the encoder. - JxlEncoderVersion()
    pub fn version(&self) -> Result<(u32, u32, u32), JxlError> {
        let version_fn: Symbol<unsafe extern "C" fn() -> u32> =
            unsafe { self.lib.get(b"JxlEncoderVersion") }.map_err(JxlError::SymbolLoadFailed)?;
        let version = unsafe { version_fn() };
        let major = version / 1_000_000;
        let minor = (version % 1_000_000) / 1_000;
        let patch = version % 1_000;

        Ok((major, minor, patch))
    }

    /// Create a new encoder. - JxlEncoderCreate()
    pub fn new(
        dll_path: &PathBuf,
        memory_manager: Option<&JxlMemoryManager>,
    ) -> Result<Self, JxlError> {
        let lib = unsafe { Library::new(dll_path) }.map_err(JxlError::LibraryLoadFailed)?;

        let create_fn: Symbol<unsafe extern "C" fn(*const JxlMemoryManager) -> *mut c_void> =
            unsafe { lib.get(b"JxlEncoderCreate") }.map_err(JxlError::SymbolLoadFailed)?;
        let enc = unsafe { create_fn(memory_manager.map_or(ptr::null(), |m| m as *const _)) };
        if enc.is_null() {
            Err(JxlError::EncoderCreationFailed)
        } else {
            Ok(JxlEncoder { lib, enc })
        }
    }

    /// Set the parallel runner function. - JxlEncoderSetParallelRunner()
    pub fn reset(&mut self) -> Result<(), JxlError> {
        let reset_fn: Symbol<unsafe extern "C" fn(*mut c_void)> = unsafe {
            self.lib
                .get(b"JxlEncoderReset")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        unsafe { reset_fn(self.enc) };
        Ok(())
    }

    pub fn set_cms(&mut self, cms: JxlCmsInterface) -> Result<(), JxlError> {
        let set_cms_fn: Symbol<unsafe extern "C" fn(*mut c_void, JxlCmsInterface)> = unsafe {
            self.lib
                .get(b"JxlEncoderSetCms")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        unsafe { set_cms_fn(self.enc, cms) };
        Ok(())
    }

    pub fn set_parallel_runner(
        &mut self,
        parallel_runner: JxlParallelRunner,
        parallel_runner_opaque: *mut c_void,
    ) -> Result<(), JxlError> {
        let set_parallel_runner_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, JxlParallelRunner, *mut c_void) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetParallelRunner")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status =
            unsafe { set_parallel_runner_fn(self.enc, parallel_runner, parallel_runner_opaque) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn get_error(&self) -> Result<JxlEncoderError, JxlError> {
        let get_error_fn: Symbol<unsafe extern "C" fn(*mut c_void) -> JxlEncoderError> = unsafe {
            self.lib
                .get(b"JxlEncoderGetError")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        Ok(unsafe { get_error_fn(self.enc) })
    }

    pub fn process_output(
        &mut self,
        next_out: &mut *mut u8,
        avail_out: &mut usize,
    ) -> Result<JxlEncoderStatus, JxlError> {
        let process_output_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, *mut *mut u8, *mut usize) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderProcessOutput")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { process_output_fn(self.enc, next_out, avail_out) };
        Ok(status)
    }

    pub fn set_frame_header(
        &self,
        frame_settings: *mut c_void,
        frame_header: &JxlFrameHeader,
    ) -> Result<(), JxlError> {
        let set_frame_header_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, *const JxlFrameHeader) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetFrameHeader")
                .map_err(JxlError::SymbolLoadFailed)?
        };

        let status = unsafe { set_frame_header_fn(frame_settings, frame_header) };

        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn set_extra_channel_blend_info(
        &self,
        frame_settings: *mut c_void,
        index: usize,
        blend_info: &JxlBlendInfo,
    ) -> Result<(), JxlError> {
        let set_extra_channel_blend_info_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, usize, *const JxlBlendInfo) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetExtraChannelBlendInfo")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { set_extra_channel_blend_info_fn(frame_settings, index, blend_info) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn set_frame_name(
        &self,
        frame_settings: *mut c_void,
        frame_name: &str,
    ) -> Result<(), JxlError> {
        let set_frame_name_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, *const c_char) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetFrameName")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let c_frame_name =
            CString::new(frame_name).map_err(|_| JxlError::InvalidInput(frame_name.to_string()))?;
        let status = unsafe { set_frame_name_fn(frame_settings, c_frame_name.as_ptr()) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn set_frame_bit_depth(
        &self,
        frame_settings: *mut c_void,
        bit_depth: &JxlBitDepth,
    ) -> Result<(), JxlError> {
        let set_frame_bit_depth_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, *const JxlBitDepth) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetFrameBitDepth")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { set_frame_bit_depth_fn(frame_settings, bit_depth) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    ///다음 프레임을 인코딩하기 위해 JPEG 인코딩된 바이트를 읽을 버퍼를 설정합니다.
    ///JxlEncoderSetBasicInfo가 아직 호출되지 않았다면, JxlEncoderAddJPEGFrame을 호출하면 추가된 JPEG 프레임의 파라미터로 암시적으로 호출됩니다.
    ///JxlEncoderSetColorEncoding 또는 JxlEncoderSetICCProfile이 아직 호출되지 않았다면, JxlEncoderAddJPEGFrame을 호출하면 추가된 JPEG 프레임의 파라미터로 암시적으로 호출됩니다.
    ///JxlEncoderStoreJPEGMetadata를 사용하여 JPEG 재구성 메타데이터를 저장하도록 인코더가 설정되어 있고 단일 JPEG 프레임이 추가된 경우, JPEG 코드스트림을 무손실로 재구성할 수 있게 됩니다.
    ///이것이 마지막 프레임인 경우, 다음 JxlEncoderProcessOutput 호출 전에 JxlEncoderCloseInput 또는 JxlEncoderCloseFrames를 호출해야 합니다.
    ///참고로, 이 함수는 무손실 압축을 위해 JPEG 프레임을 추가하는 데에만 사용할 수 있습니다. 손실 압축으로 인코딩하려면 JPEG을 수동으로 디코딩하고 JxlEncoderAddImageFrame을 사용하여 픽셀 버퍼를 추가해야 합니다.
    ///
    ///
    /// 요약!!!! jpg 파일을 jxl로 인코딩할 때, add_jpeg_frame만 호출하면 JxlEncoderSetBasicInfo, JxlEncoderSetICCProfile, JxlEncoderSetColorEncoding을 호출하지 않아도 된다
    pub fn add_jpeg_frame(
        &self,
        frame_settings: *const c_void,
        buffer: &[u8],
    ) -> Result<(), JxlError> {
        let add_jpeg_frame_fn: Symbol<
            unsafe extern "C" fn(*const c_void, *const u8, usize) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderAddJPEGFrame")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { add_jpeg_frame_fn(frame_settings, buffer.as_ptr(), buffer.len()) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn add_image_frame(
        &self,
        frame_settings: *mut c_void,
        pixel_format: &JxlPixelFormat,
        buffer: &[u8],
    ) -> Result<(), JxlError> {
        let add_image_frame_fn: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                *const JxlPixelFormat,
                *const u8,
                usize,
            ) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderAddImageFrame")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe {
            add_image_frame_fn(frame_settings, pixel_format, buffer.as_ptr(), buffer.len())
        };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn set_output_processor(
        &mut self,
        output_processor: JxlEncoderOutputProcessor,
    ) -> Result<(), JxlError> {
        let set_output_processor_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, JxlEncoderOutputProcessor) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetOutputProcessor")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { set_output_processor_fn(self.enc, output_processor) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn flush_input(&mut self) -> Result<(), JxlError> {
        let flush_input_fn: Symbol<unsafe extern "C" fn(*mut c_void) -> JxlEncoderStatus> = unsafe {
            self.lib
                .get(b"JxlEncoderFlushInput")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { flush_input_fn(self.enc) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn add_chunked_frame(
        &self,
        frame_settings: *mut c_void,
        is_last_frame: bool,
        chunked_frame_input: JxlChunkedFrameInputSource,
    ) -> Result<(), JxlError> {
        let add_chunked_frame_fn: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                JxlBool,
                JxlChunkedFrameInputSource,
            ) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderAddChunkedFrame")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe {
            add_chunked_frame_fn(
                frame_settings,
                if is_last_frame {
                    JxlBool::True
                } else {
                    JxlBool::False
                },
                chunked_frame_input,
            )
        };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn set_extra_channel_buffer(
        &self,
        frame_settings: *mut c_void,
        pixel_format: &JxlPixelFormat,
        buffer: &[u8],
        index: u32,
    ) -> Result<(), JxlError> {
        let set_extra_channel_buffer_fn: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                *const JxlPixelFormat,
                *const u8,
                usize,
                u32,
            ) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetExtraChannelBuffer")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe {
            set_extra_channel_buffer_fn(
                frame_settings,
                pixel_format,
                buffer.as_ptr(),
                buffer.len(),
                index,
            )
        };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn add_box(
        &mut self,
        type_: &JxlBoxType,
        contents: &[u8],
        compress_box: bool,
    ) -> Result<(), JxlError> {
        let add_box_fn: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                *const JxlBoxType,
                *const u8,
                usize,
                JxlBool,
            ) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderAddBox")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe {
            add_box_fn(
                self.enc,
                type_,
                contents.as_ptr(),
                contents.len(),
                if compress_box {
                    JxlBool::True
                } else {
                    JxlBool::False
                },
            )
        };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn use_boxes(&mut self) -> Result<(), JxlError> {
        let use_boxes_fn: Symbol<unsafe extern "C" fn(*mut c_void) -> JxlEncoderStatus> = unsafe {
            self.lib
                .get(b"JxlEncoderUseBoxes")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { use_boxes_fn(self.enc) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn close_boxes(&mut self) -> Result<(), JxlError> {
        let close_boxes_fn: Symbol<unsafe extern "C" fn(*mut c_void)> = unsafe {
            self.lib
                .get(b"JxlEncoderCloseBoxes")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        unsafe { close_boxes_fn(self.enc) };
        Ok(())
    }

    pub fn close_frames(&mut self) -> Result<(), JxlError> {
        let close_frames_fn: Symbol<unsafe extern "C" fn(*mut c_void)> = unsafe {
            self.lib
                .get(b"JxlEncoderCloseFrames")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        unsafe { close_frames_fn(self.enc) };
        Ok(())
    }

    pub fn close_input(&mut self) -> Result<(), JxlError> {
        let close_input_fn: Symbol<unsafe extern "C" fn(*mut c_void)> = unsafe {
            self.lib
                .get(b"JxlEncoderCloseInput")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        unsafe { close_input_fn(self.enc) };
        Ok(())
    }

    pub fn set_color_encoding(&mut self, color: &JxlColorEncoding) -> Result<(), JxlError> {
        let set_color_encoding_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, *const JxlColorEncoding) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetColorEncoding")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { set_color_encoding_fn(self.enc, color) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::ColorProfileError),
        }
    }

    pub fn set_icc_profile(&mut self, icc_profile: &[u8]) -> Result<(), JxlError> {
        let set_icc_profile_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, *const u8, usize) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetICCProfile")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status =
            unsafe { set_icc_profile_fn(self.enc, icc_profile.as_ptr(), icc_profile.len()) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::ColorProfileError),
        }
    }
    ///JxlBasicInfo 구조체를 기본값으로 초기화합니다.
    /// 전방 호환성을 위해 이 함수는 구조체 필드에 값을 할당하기 전에 호출해야 합니다.
    /// 기본값은 8비트 RGB 이미지에 해당하며 알파나 다른 추가 채널은 없습니다.
    pub fn init_basic_info(&self, info: &mut JxlBasicInfo) -> Result<(), JxlError> {
        let init_basic_info_fn: Symbol<unsafe extern "C" fn(*mut JxlBasicInfo)> = unsafe {
            self.lib
                .get(b"JxlEncoderInitBasicInfo")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        unsafe { init_basic_info_fn(info) };
        Ok(())
    }

    pub fn init_frame_header(&self, frame_header: &mut JxlFrameHeader) -> Result<(), JxlError> {
        let init_frame_header_fn: Symbol<unsafe extern "C" fn(*mut JxlFrameHeader)> = unsafe {
            self.lib
                .get(b"JxlEncoderInitFrameHeader")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        unsafe { init_frame_header_fn(frame_header) };
        Ok(())
    }

    pub fn init_blend_info(&self, blend_info: &mut JxlBlendInfo) -> Result<(), JxlError> {
        let init_blend_info_fn: Symbol<unsafe extern "C" fn(*mut JxlBlendInfo)> = unsafe {
            self.lib
                .get(b"JxlEncoderInitBlendInfo")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        unsafe { init_blend_info_fn(blend_info) };
        Ok(())
    }

    pub fn set_basic_info(&mut self, info: &JxlBasicInfo) -> Result<(), JxlError> {
        let set_basic_info_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, *const JxlBasicInfo) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetBasicInfo")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { set_basic_info_fn(self.enc, info) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn set_upsampling_mode(&mut self, factor: i64, mode: i64) -> Result<(), JxlError> {
        let set_upsampling_mode_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, i64, i64) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetUpsamplingMode")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { set_upsampling_mode_fn(self.enc, factor, mode) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn init_extra_channel_info(
        &self,
        type_: JxlExtraChannelType,
        info: &mut JxlExtraChannelInfo,
    ) -> Result<(), JxlError> {
        let init_extra_channel_info_fn: Symbol<
            unsafe extern "C" fn(JxlExtraChannelType, *mut JxlExtraChannelInfo),
        > = unsafe {
            self.lib
                .get(b"JxlEncoderInitExtraChannelInfo")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        unsafe { init_extra_channel_info_fn(type_, info) };
        Ok(())
    }

    pub fn set_extra_channel_info(
        &mut self,
        index: usize,
        info: &JxlExtraChannelInfo,
    ) -> Result<(), JxlError> {
        let set_extra_channel_info_fn: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                usize,
                *const JxlExtraChannelInfo,
            ) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetExtraChannelInfo")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { set_extra_channel_info_fn(self.enc, index, info) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn set_extra_channel_name(&mut self, index: usize, name: &str) -> Result<(), JxlError> {
        let set_extra_channel_name_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, usize, *const c_char, usize) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetExtraChannelName")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let c_name = CString::new(name).map_err(|_| JxlError::InvalidInput(name.to_string()))?;
        let status =
            unsafe { set_extra_channel_name_fn(self.enc, index, c_name.as_ptr(), name.len()) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn use_container(&mut self, use_container: bool) -> Result<(), JxlError> {
        let use_container_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, JxlBool) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderUseContainer")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe {
            use_container_fn(
                self.enc,
                if use_container {
                    JxlBool::True
                } else {
                    JxlBool::False
                },
            )
        };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn store_jpeg_metadata(&mut self, store_jpeg_metadata: bool) -> Result<(), JxlError> {
        let store_jpeg_metadata_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, JxlBool) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderStoreJPEGMetadata")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe {
            store_jpeg_metadata_fn(
                self.enc,
                if store_jpeg_metadata {
                    JxlBool::True
                } else {
                    JxlBool::False
                },
            )
        };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn set_codestream_level(&mut self, level: c_int) -> Result<(), JxlError> {
        let set_codestream_level_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, c_int) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetCodestreamLevel")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { set_codestream_level_fn(self.enc, level) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn get_required_codestream_level(&self) -> Result<c_int, JxlError> {
        let get_required_codestream_level_fn: Symbol<unsafe extern "C" fn(*const c_void) -> c_int> = unsafe {
            self.lib
                .get(b"JxlEncoderGetRequiredCodestreamLevel")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        Ok(unsafe { get_required_codestream_level_fn(self.enc) })
    }

    pub fn distance_from_quality(&self, quality: f32) -> Result<f32, JxlError> {
        let distance_from_quality_fn: Symbol<unsafe extern "C" fn(f32) -> f32> = unsafe {
            self.lib
                .get(b"JxlEncoderDistanceFromQuality")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        Ok(unsafe { distance_from_quality_fn(quality) })
    }

    pub fn color_encoding_set_to_srgb(
        &self,
        color_encoding: &mut JxlColorEncoding,
        is_gray: bool,
    ) -> Result<(), JxlError> {
        let color_encoding_set_to_srgb_fn: Symbol<
            unsafe extern "C" fn(*mut JxlColorEncoding, JxlBool),
        > = unsafe {
            self.lib
                .get(b"JxlColorEncodingSetToSRGB")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        unsafe {
            color_encoding_set_to_srgb_fn(
                color_encoding,
                if is_gray {
                    JxlBool::True
                } else {
                    JxlBool::False
                },
            )
        };
        Ok(())
    }

    pub fn color_encoding_set_to_linear_srgb(
        &self,
        color_encoding: &mut JxlColorEncoding,
        is_gray: bool,
    ) -> Result<(), JxlError> {
        let color_encoding_set_to_linear_srgb_fn: Symbol<
            unsafe extern "C" fn(*mut JxlColorEncoding, JxlBool),
        > = unsafe {
            self.lib
                .get(b"JxlColorEncodingSetToLinearSRGB")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        unsafe {
            color_encoding_set_to_linear_srgb_fn(
                color_encoding,
                if is_gray {
                    JxlBool::True
                } else {
                    JxlBool::False
                },
            )
        };
        Ok(())
    }

    pub fn allow_expert_options(&mut self) -> Result<(), JxlError> {
        let allow_expert_options_fn: Symbol<unsafe extern "C" fn(*mut c_void)> = unsafe {
            self.lib
                .get(b"JxlEncoderAllowExpertOptions")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        unsafe { allow_expert_options_fn(self.enc) };
        Ok(())
    }

    /// 새로운 인코더 옵션 세트를 생성하며, 모든 값은 초기에 `source` 옵션에서 복사되거나 `source`가 NULL인 경우 기본값으로 설정됩니다.
    /// 반환된 포인터는 인코더에 연결된 불투명 구조체이며, JxlEncoderDestroy()가 호출될 때 인코더에 의해 할당 해제됩니다.
    /// JxlEncoder와 JxlEncoderFrameSettings를 모두 사용하는 함수의 경우, 동일한 인코더 인스턴스에 대해 이 함수로 생성된 JxlEncoderFrameSettings만 사용할 수 있습니다.
    /// source가 Some이면 (즉, 널이 아닌 포인터가 제공되면):
    /// 새로운 프레임 설정이 생성되고, 모든 값이 source가 가리키는 기존 프레임 설정에서 복사됩니다.
    /// 이를 통해 기존 설정을 기반으로 새로운 설정을 만들 수 있습니다.
    /// source가 None이면 (즉, 널 포인터가 제공되면):
    /// 새로운 프레임 설정이 생성되고, 모든 값이 기본값으로 초기화됩니다.
    /// 이는 완전히 새로운 기본 설정으로 시작하고 싶을 때 유용합니다.
    pub fn create_frame_settings(
        &self,
        source: Option<*const c_void>,
    ) -> Result<*mut c_void, JxlError> {
        let create_fn: Symbol<unsafe extern "C" fn(*mut c_void, *const c_void) -> *mut c_void> = unsafe {
            self.lib
                .get(b"JxlEncoderFrameSettingsCreate")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let settings = unsafe { create_fn(self.enc, source.unwrap_or(std::ptr::null())) };
        if settings.is_null() {
            Err(JxlError::EncoderCreationFailed)
        } else {
            Ok(settings)
        }
    }

    pub fn set_frame_option(
        &self,
        settings: *mut c_void,
        option: JxlEncoderFrameSettingId,
        value: i64,
    ) -> Result<(), JxlError> {
        let set_option_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, JxlEncoderFrameSettingId, i64) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderFrameSettingsSetOption")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { set_option_fn(settings, option, value) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn set_frame_float_option(
        &self,
        settings: *mut c_void,
        option: JxlEncoderFrameSettingId,
        value: f32,
    ) -> Result<(), JxlError> {
        let set_float_option_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, JxlEncoderFrameSettingId, f32) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderFrameSettingsSetFloatOption")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { set_float_option_fn(settings, option, value) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    ///무손실 인코딩을 활성화합니다.
    /// 이것은 그 자체로 다른 옵션들과 같은 단순한 옵션이 아닙니다.
    /// 대신 활성화되면 기존의 여러 옵션들(예: 거리, 모듈러 모드, 색상 변환 등)을 재정의하여 비트 단위의 무손실 인코딩을 가능하게 합니다.
    /// 비활성화된 경우, 이러한 옵션들은 재정의되지 않습니다.
    /// 하지만 이 옵션들이 수동으로 무손실 작동 조합으로 설정될 수 있기 때문에, 이 함수를 JXL_FALSE로 설정하여 사용하는 것이 반드시 손실 인코딩을 보장하지는 않습니다.
    /// 다만, 기본 옵션 세트는 손실 인코딩입니다.
    /// not working why???
    pub fn set_frame_lossless(
        &self,
        settings: *mut c_void,
        lossless: bool,
    ) -> Result<(), JxlError> {
        let set_lossless_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, JxlBool) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetFrameLossless")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe {
            set_lossless_fn(
                settings,
                if lossless {
                    JxlBool::True
                } else {
                    JxlBool::False
                },
            )
        };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    /// 손실 압축을 위한 거리 레벨을 설정합니다: 목표 최대 butteraugli 거리, 낮을수록 더 높은 품질을 의미합니다.
    /// 범위: 0 .. 25. 0.0 = 수학적으로 무손실 (그러나 진정한 무손실을 위해서는 JxlEncoderSetFrameLossless를 대신 사용하세요.
    /// 거리를 0으로 설정하는 것만으로는 무손실의 유일한 요구사항이 아닙니다).
    /// 1.0 = 시각적으로 무손실. 권장 범위: 0.5 .. 3.0. 기본값: 1.0.
    pub fn set_frame_distance(&self, settings: *mut c_void, distance: f32) -> Result<(), JxlError> {
        let set_distance_fn: Symbol<unsafe extern "C" fn(*mut c_void, f32) -> JxlEncoderStatus> = unsafe {
            self.lib
                .get(b"JxlEncoderSetFrameDistance")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { set_distance_fn(settings, distance) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }

    pub fn set_frame_extra_channel_distance(
        &self,
        settings: *mut c_void,
        index: usize,
        distance: f32,
    ) -> Result<(), JxlError> {
        let set_extra_channel_distance_fn: Symbol<
            unsafe extern "C" fn(*mut c_void, usize, f32) -> JxlEncoderStatus,
        > = unsafe {
            self.lib
                .get(b"JxlEncoderSetExtraChannelDistance")
                .map_err(JxlError::SymbolLoadFailed)?
        };
        let status = unsafe { set_extra_channel_distance_fn(settings, index, distance) };
        match status {
            JxlEncoderStatus::Success => Ok(()),
            _ => Err(JxlError::from_encoder_status(status, self.get_error().ok())),
        }
    }
}

impl Drop for JxlEncoder {
    fn drop(&mut self) {
        let destroy_fn: Symbol<unsafe extern "C" fn(*mut c_void)> = unsafe {
            self.lib
                .get(b"JxlEncoderDestroy")
                .expect("Failed to load JxlEncoderDestroy")
        };
        unsafe { destroy_fn(self.enc) };
    }
}
