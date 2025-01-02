use std::ffi::{c_int, c_void};

/// Type for parallel runner function.
pub type JxlParallelRunner = unsafe extern "C" fn(*mut c_void, *mut c_void) -> c_int;

/// Represents the size of a box in the JPEG XL container.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JxlBoxContentSizeRaw {
    pub size: u64,
    pub compressed: bool,
}

/// Represents frame index information.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct JxlFrameIndex {
    pub offset: u64,
    pub size: u64,
}
