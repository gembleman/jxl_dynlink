use crate::error::JxlError;
use crate::{JxlBool, JxlGainMapBundle, JxlMemoryManager};
use libloading::{Library, Symbol};

use std::ptr;

/// ICC 프로필과 GainMap 관련 기능을 처리하는 구조체
#[derive(Debug)]
pub struct JxlIccGainMap {
    lib: Library,
}

impl JxlIccGainMap {
    /// 새로운 JxlIccGainMap 인스턴스를 생성합니다.
    pub fn new<P: AsRef<std::ffi::OsStr>>(path: P) -> Result<Self, JxlError> {
        let lib = unsafe { Library::new(path) }.map_err(JxlError::LibraryLoadFailed)?;

        Ok(Self { lib })
    }

    /// ICC 프로필을 압축합니다.
    pub fn encode_icc_profile(
        &self,
        memory_manager: &JxlMemoryManager,
        icc_data: &[u8],
    ) -> Result<Vec<u8>, JxlError> {
        let encode_fn: Symbol<
            unsafe extern "C" fn(
                *const JxlMemoryManager,
                *const u8,
                usize,
                *mut *mut u8,
                *mut usize,
            ) -> JxlBool,
        > = unsafe { self.lib.get(b"JxlICCProfileEncode") }.map_err(JxlError::SymbolLoadFailed)?;

        let mut compressed_icc: *mut u8 = ptr::null_mut();
        let mut compressed_size: usize = 0;

        let result = unsafe {
            encode_fn(
                memory_manager,
                icc_data.as_ptr(),
                icc_data.len(),
                &mut compressed_icc,
                &mut compressed_size,
            )
        };

        if result == JxlBool::False || compressed_icc.is_null() {
            return Err(JxlError::ICCProfileEncodeFailed(
                "Failed to encode ICC profile".to_string(),
            ));
        }

        unsafe {
            let compressed_data =
                std::slice::from_raw_parts(compressed_icc, compressed_size).to_vec();
            // 메모리 해제는 memory_manager를 통해 수행되어야 합니다
            Ok(compressed_data)
        }
    }

    /// 압축된 ICC 프로필을 디코딩합니다.
    pub fn decode_icc_profile(
        &self,
        memory_manager: &JxlMemoryManager,
        compressed_data: &[u8],
    ) -> Result<Vec<u8>, JxlError> {
        let decode_fn: Symbol<
            unsafe extern "C" fn(
                *const JxlMemoryManager,
                *const u8,
                usize,
                *mut *mut u8,
                *mut usize,
            ) -> JxlBool,
        > = unsafe { self.lib.get(b"JxlICCProfileDecode") }.map_err(JxlError::SymbolLoadFailed)?;

        let mut icc: *mut u8 = ptr::null_mut();
        let mut icc_size: usize = 0;

        let result = unsafe {
            decode_fn(
                memory_manager,
                compressed_data.as_ptr(),
                compressed_data.len(),
                &mut icc,
                &mut icc_size,
            )
        };

        if result == JxlBool::False || icc.is_null() {
            return Err(JxlError::ICCProfileDecodeFailed(
                "Failed to decode ICC profile".to_string(),
            ));
        }

        unsafe {
            let decoded_data = std::slice::from_raw_parts(icc, icc_size).to_vec();
            // 메모리 해제는 memory_manager를 통해 수행되어야 합니다
            Ok(decoded_data)
        }
    }

    /// 게인맵 번들의 직렬화에 필요한 전체 크기를 계산합니다.
    pub fn get_gain_map_bundle_size(
        &self,
        map_bundle: &JxlGainMapBundle,
    ) -> Result<usize, JxlError> {
        let get_size_fn: Symbol<
            unsafe extern "C" fn(*const JxlGainMapBundle, *mut usize) -> JxlBool,
        > = unsafe { self.lib.get(b"JxlGainMapGetBundleSize") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let mut bundle_size: usize = 0;
        let result = unsafe { get_size_fn(map_bundle, &mut bundle_size) };

        if result == JxlBool::False {
            return Err(JxlError::GainMapBundleSizeFailed);
        }

        Ok(bundle_size)
    }

    /// 게인맵 번들을 버퍼에 직렬화합니다.
    pub fn write_gain_map_bundle(
        &self,
        map_bundle: &JxlGainMapBundle,
        output_buffer: &mut [u8],
    ) -> Result<usize, JxlError> {
        let write_fn: Symbol<
            unsafe extern "C" fn(*const JxlGainMapBundle, *mut u8, usize, *mut usize) -> JxlBool,
        > = unsafe { self.lib.get(b"JxlGainMapWriteBundle") }
            .map_err(JxlError::SymbolLoadFailed)?;

        let mut bytes_written: usize = 0;
        let result = unsafe {
            write_fn(
                map_bundle,
                output_buffer.as_mut_ptr(),
                output_buffer.len(),
                &mut bytes_written,
            )
        };

        if result == JxlBool::False {
            return Err(JxlError::GainMapBundleWriteFailed(
                "Failed to write gain map bundle".to_string(),
            ));
        }

        Ok(bytes_written)
    }

    /// 버퍼로부터 게인맵 번들을 역직렬화합니다.
    pub fn read_gain_map_bundle(
        &self,
        map_bundle: &mut JxlGainMapBundle,
        input_buffer: &[u8],
    ) -> Result<usize, JxlError> {
        let read_fn: Symbol<
            unsafe extern "C" fn(*mut JxlGainMapBundle, *const u8, usize, *mut usize) -> JxlBool,
        > = unsafe { self.lib.get(b"JxlGainMapReadBundle") }.map_err(JxlError::SymbolLoadFailed)?;

        let mut bytes_read: usize = 0;
        let result = unsafe {
            read_fn(
                map_bundle,
                input_buffer.as_ptr(),
                input_buffer.len(),
                &mut bytes_read,
            )
        };

        if result == JxlBool::False {
            return Err(JxlError::GainMapBundleReadFailed(
                "Failed to read gain map bundle".to_string(),
            ));
        }

        Ok(bytes_read)
    }
}

impl Drop for JxlIccGainMap {
    fn drop(&mut self) {
        // Library는 자동으로 해제됩니다
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icc_gainmap_creation() {
        let lib_path = "dlls/jxl.dll"; // 실제 라이브러리 경로로 변경 필요
        let result = JxlIccGainMap::new(lib_path);
        if result.is_err() {
            println!("{:?}", result.unwrap_err());
            return;
        }
    }

    // 추가 테스트 케이스들...
}
