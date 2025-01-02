use crate::common::JxlPixelFormat;
use std::os::raw::c_void;

#[repr(C)]
pub struct JxlEncoderOutputProcessor {
    pub opaque: *mut c_void,
    pub get_buffer:
        Option<unsafe extern "C" fn(opaque: *mut c_void, size: *mut usize) -> *mut c_void>,
    pub release_buffer: Option<unsafe extern "C" fn(opaque: *mut c_void, written_bytes: usize)>,
    pub seek: Option<unsafe extern "C" fn(opaque: *mut c_void, position: u64)>,
    pub set_finalized_position:
        Option<unsafe extern "C" fn(opaque: *mut c_void, finalized_position: u64)>,
}

#[repr(C)]
pub struct JxlChunkedFrameInputSource {
    pub opaque: *mut c_void,
    pub get_color_channels_pixel_format:
        Option<unsafe extern "C" fn(opaque: *mut c_void, pixel_format: *mut JxlPixelFormat)>,
    pub get_color_channel_data_at: Option<
        unsafe extern "C" fn(
            opaque: *mut c_void,
            xpos: usize,
            ypos: usize,
            xsize: usize,
            ysize: usize,
            row_offset: *mut usize,
        ) -> *const c_void,
    >,
    pub get_extra_channel_pixel_format: Option<
        unsafe extern "C" fn(
            opaque: *mut c_void,
            ec_index: usize,
            pixel_format: *mut JxlPixelFormat,
        ),
    >,
    pub get_extra_channel_data_at: Option<
        unsafe extern "C" fn(
            opaque: *mut c_void,
            ec_index: usize,
            xpos: usize,
            ypos: usize,
            xsize: usize,
            ysize: usize,
            row_offset: *mut usize,
        ) -> *const c_void,
    >,
    pub release_buffer: Option<unsafe extern "C" fn(opaque: *mut c_void, buf: *const c_void)>,
}
