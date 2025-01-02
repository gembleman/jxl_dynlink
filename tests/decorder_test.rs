use jxl_dynlink::*;
use std::{
    error::Error,
    ffi::{c_int, c_void},
    fs::File,
    io::Read,
    ptr,
};

const DLL_PATH: &str = "dlls/jxl.dll";

#[test]
fn test_decoder_creation_and_version() -> Result<(), JxlError> {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    let (major, minor, patch) = decoder.version()?;
    println!("JxlDecoder version: {}.{}.{}", major, minor, patch);
    Ok(())
}

#[test]
fn test_signature_check() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");

    // Valid JPEG XL signature (container format)
    let valid_jxl_data = [
        0x00, 0x00, 0x00, 0x0C, 0x4A, 0x58, 0x4C, 0x20, 0x0D, 0x0A, 0x87, 0x0A, 0x00, 0x00, 0x00,
        0x14, 0x66, 0x74, 0x79, 0x70, 0x6A, 0x78, 0x6C, 0x20,
    ];
    let result = decoder.check_signature(&valid_jxl_data);
    assert!(
        matches!(result, Ok(JxlSignature::Container)),
        "Expected Container signature, got {:?}",
        result
    );

    // Valid JPEG XL signature (codestream format)
    let valid_codestream_data = [0xFF, 0x0A];
    let result = decoder.check_signature(&valid_codestream_data);
    assert!(
        matches!(result, Ok(JxlSignature::Codestream)),
        "Expected Codestream signature, got {:?}",
        result
    );

    // Invalid data - I dont know make invalid data, so pass

    // Not enough bytes
    let not_enough_data = [0x00];
    let result = decoder.check_signature(&not_enough_data);
    assert!(
        matches!(result, Ok(JxlSignature::NotEnoughBytes)),
        "Expected NotEnoughBytes, got {:?}",
        result
    );
}

#[test]
fn test_decoder_reset_and_rewind() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    decoder.reset();
    decoder.rewind();
    // These functions don't return anything, so we just ensure they don't panic
}

#[test]
fn test_frame_operations() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");

    let buffer = read_jxl_file("test_imgs/test.jxl").expect("Failed to read test.jxl");
    assert!(decoder.set_input(&buffer).is_ok());
    assert!(decoder.process_input().is_ok());

    let frame_name = decoder.get_frame_name();
    if frame_name.is_err() {
        // If we can't get frame name, we can't set it either
        println!("Failed to get frame name: {:?}", frame_name);
        return;
    }
}

#[test]
fn test_parallel_runner() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");

    unsafe extern "C" fn dummy_runner(
        _opaque_runner: *mut c_void,
        _opaque_jpegxl: *mut c_void,
    ) -> c_int {
        0 // Just a dummy implementation
    }

    assert!(decoder
        .set_parallel_runner(Some(dummy_runner), ptr::null_mut())
        .is_ok());
}

#[test]
fn test_size_hint_and_events() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    let hint = decoder.size_hint_basic_info();
    assert!(hint > 0, "Size hint should be greater than 0");

    assert!(decoder
        .subscribe_events(JxlDecoderStatus::BasicInfo | JxlDecoderStatus::ColorEncoding)
        .is_ok());
}

#[test]
fn test_orientation_and_alpha_settings() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    assert!(decoder.set_keep_orientation(true).is_ok());
    assert!(decoder.set_unpremultiply_alpha(true).is_ok());
    assert!(decoder.set_render_spotcolors(false).is_ok());
    assert!(decoder.set_coalescing(true).is_ok());
}

#[test]
fn test_input_processing() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    let dummy_input = vec![0xFF, 0x0A]; // Dummy JPEG XL data
    assert!(decoder.set_input(&dummy_input).is_ok());

    match decoder.process_input() {
        Ok(status) => println!("Process input status: {:?}", status),
        Err(e) => println!("Process input error: {:?}", e),
    }

    let remaining = decoder.release_input();
    println!("Remaining unprocessed bytes: {}", remaining);

    decoder.close_input();
}

#[test]
fn test_basic_info_and_color_encoding() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    // We need to set some input and process it before we can get basic info
    let dummy_input = vec![0xFF, 0x0A]; // Dummy JPEG XL data
    decoder
        .set_input(&dummy_input)
        .expect("Failed to set input");
    decoder.process_input().expect("Failed to process input");

    match decoder.get_basic_info() {
        Ok(info) => println!("Basic info: {:?}", info),
        Err(e) => println!("Failed to get basic info: {:?}", e),
    }

    match decoder.get_color_as_encoded_profile(JxlColorProfileTarget::Data) {
        Ok(profile) => println!("Color profile: {:?}", profile),
        Err(e) => println!("Failed to get color profile: {:?}", e),
    }
}

#[test]
fn test_extra_channel_info() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    // Assuming we've processed enough input to have extra channel info
    match decoder.get_extra_channel_info(0) {
        Ok(info) => println!("Extra channel info: {:?}", info),
        Err(e) => println!("Failed to get extra channel info: {:?}", e),
    }

    match decoder.get_extra_channel_name(0) {
        Ok(name) => println!("Extra channel name: {}", name),
        Err(e) => println!("Failed to get extra channel name: {:?}", e),
    }
}

#[test]
fn test_icc_profile() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    // Assuming we've processed enough input to have ICC profile info
    let mut size: usize = 0;
    match decoder.get_icc_profile_size(JxlColorProfileTarget::Original, Some(&mut size)) {
        Ok(_) => println!("ICC profile size: {:?}", size),
        Err(e) => println!("Failed to get ICC profile size: {:?}", e),
    }
    let mut profile_icc = vec![0u8; size];
    match decoder.get_color_as_icc_profile(JxlColorProfileTarget::Data, &mut profile_icc, size) {
        Ok(_) => println!("ICC profile data length: {}", profile_icc.len()),
        Err(e) => println!("Failed to get ICC profile: {:?}", e),
    }
}

//need jxl file with icc profile
#[test]
fn test_color_management() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    let file = read_jxl_file("test_imgs/test.jxl").expect("Failed to read test.jxl");
    decoder.set_input(&file).expect("Failed to set input");
    decoder.process_input().expect("Failed to process input");

    let color_encoding = decoder.get_color_as_encoded_profile(JxlColorProfileTarget::Data);
    if color_encoding.is_err() {
        // If we can't get color profile, we can't set it either
        println!("Failed to get color profile: {:?}", color_encoding);
        return;
    }

    // let color_encoding = JxlColorEncoding::default();
    assert!(decoder
        .set_preferred_color_profile(&color_encoding.unwrap())
        .is_ok());
    assert!(decoder.set_desired_intensity_target(100.0).is_ok());

    let cms = JxlCmsInterface::default();
    assert!(decoder.set_cms(&cms).is_ok());
}

//need jxl file with metadata
#[test]
fn test_preview_and_frame_info() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    let file = read_jxl_file("test_imgs/test.jxl").expect("Failed to read test.jxl");
    decoder.set_input(&file).expect("Failed to set input");
    decoder.process_input().expect("Failed to process input");
    let basic_info = decoder.get_basic_info().expect("Failed to get basic info");

    let format = create_pixel_format_from_basic_info(&basic_info);

    match decoder.preview_out_buffer_size(&format) {
        Ok(size) => println!("Preview buffer size: {}", size),
        Err(e) => println!("Failed to get preview buffer size: {:?}", e),
    }
    return ();

    let mut buffer = vec![0u8; 1024]; // Dummy buffer
    assert!(decoder.set_preview_out_buffer(&format, &mut buffer).is_ok());

    match decoder.get_frame_header() {
        Ok(header) => println!("Frame header: {:?}", header),
        Err(e) => println!("Failed to get frame header: {:?}", e),
    }

    match decoder.get_frame_name() {
        Ok(name) => println!("Frame name: {}", name),
        Err(e) => println!("Failed to get frame name: {:?}", e),
    }

    match decoder.get_extra_channel_blend_info(0) {
        Ok(info) => println!("Extra channel blend info: {:?}", info),
        Err(e) => println!("Failed to get extra channel blend info: {:?}", e),
    }
}

//need jxl file with metadata
#[test]
fn test_image_out_buffer() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    let format = JxlPixelFormat::default();

    match decoder.image_out_buffer_size(&format) {
        Ok(size) => println!("Image out buffer size: {}", size),
        Err(e) => println!("Failed to get image out buffer size: {:?}", e),
    }

    let mut buffer = vec![0u8; 1024]; // Dummy buffer
    assert!(decoder.set_image_out_buffer(&format, &mut buffer).is_ok());

    // Test image out callback
    decoder
        .set_image_out_callback(&format, |_, x, y, num_pixels, _| {
            println!(
                "Callback called for region: x={}, y={}, num_pixels={}",
                x, y, num_pixels
            );
        })
        .expect("Failed to set image out callback");

    // Test multithreaded image out callback
    decoder
        .set_multithreaded_image_out_callback(
            &format,
            |_, num_threads, num_pixels_per_thread| {
                println!(
                    "Init callback: num_threads={}, num_pixels_per_thread={}",
                    num_threads, num_pixels_per_thread
                );
                Box::into_raw(Box::new(())) as *mut c_void
            },
            |_, thread_id, x, y, num_pixels, _| {
                println!(
                    "Run callback: thread_id={}, x={}, y={}, num_pixels={}",
                    thread_id, x, y, num_pixels
                );
            },
            Some(|_| println!("Destroy callback called")),
            ptr::null_mut(),
        )
        .expect("Failed to set multithreaded image out callback");
}

//need jxl with extra channel
#[test]
fn test_extra_channel_buffer() {
    return ();
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    let format = JxlPixelFormat::default();

    match decoder.extra_channel_buffer_size(&format, 0) {
        Ok(size) => println!("Extra channel buffer size: {}", size),
        Err(e) => println!("Failed to get extra channel buffer size: {:?}", e),
    }

    let mut buffer = vec![0u8; 1024]; // Dummy buffer
    assert!(decoder
        .set_extra_channel_buffer(&format, &mut buffer, 0)
        .is_ok());
}

#[test]
fn test_jpeg_reconstruction() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    let mut buffer = vec![0u8; 1024]; // Dummy buffer
    assert!(decoder.set_jpeg_buffer(&mut buffer).is_ok());
    let remaining = decoder.release_jpeg_buffer();
    println!("Remaining JPEG buffer size: {}", remaining);
}

#[test]
fn test_box_operations() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");

    let mut buffer = read_jxl_file("test_imgs/test.jxl").expect("Failed to read test.jxl");

    // Set box buffer - pass

    // assert!(decoder.set_box_buffer(&mut buffer).is_ok());
    // let remaining = decoder.release_box_buffer();
    // println!("Remaining box buffer size: {}", remaining);

    // assert!(decoder.set_decompress_boxes(true).is_ok());

    // match decoder.get_box_type() {
    //     Ok(box_type) => println!("Box type: {:?}", box_type),
    //     Err(e) => println!("Failed to get box type: {:?}", e),
    // }

    // match decoder.get_box_size_raw() {
    //     Ok(size) => println!("Raw box size: {}", size),
    //     Err(e) => println!("Failed to get raw box size: {:?}", e),
    // }

    // match decoder.get_box_size_contents() {
    //     Ok(size) => println!("Box contents size: {}", size),
    //     Err(e) => println!("Failed to get box contents size: {:?}", e),
    // }
}

#[test]
fn test_progressive_decoding() {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");
    let file = read_jxl_file("test_imgs/test.jxl").expect("Failed to read test.jxl");
    decoder.set_input(&file).expect("Failed to set input");
    decoder.process_input().expect("Failed to process input");

    assert!(decoder
        .set_progressive_detail(JxlProgressiveDetail::KDc)
        .is_ok());

    let ratio = decoder.get_intended_downsampling_ratio();
    println!("Intended downsampling ratio: {}", ratio);

    // assert!(decoder.flush_image().is_ok());
}

#[test]
fn test_image_out_bit_depth() -> Result<(), JxlError> {
    let decoder = JxlDecoder::new(DLL_PATH).expect("Failed to create decoder");

    // JPEG XL 파일 읽기
    let mut file = File::open("test_imgs/test.jxl").expect("Failed to open test.jxl");
    let mut jxl_data = Vec::new();
    file.read_to_end(&mut jxl_data)
        .expect("Failed to read test.jxl");

    // 이벤트 구독
    decoder.subscribe_events(JxlDecoderStatus::BasicInfo | JxlDecoderStatus::Frame)?;

    // 입력 설정
    decoder.set_input(&jxl_data).expect("Failed to set input");

    let mut basic_info = None;
    let mut frame_header = None;

    // 디코딩 프로세스 실행
    loop {
        println!("Processing input");
        match decoder.process_input() {
            Ok(JxlDecoderStatus::BasicInfo) => {
                println!("Basic info event received");
                basic_info = Some(decoder.get_basic_info().expect("Failed to get basic info"));
                println!("Basic info: {:?}", basic_info.as_ref().unwrap());
                break; // 기본 정보를 얻었으므로 루프 종료
            }
            Ok(JxlDecoderStatus::Frame) => {
                frame_header = Some(
                    decoder
                        .get_frame_header()
                        .expect("Failed to get frame header"),
                );
                println!("Frame header: {:?}", frame_header.as_ref().unwrap());
                break; // 프레임 헤더를 얻었으므로 루프 종료
            }
            Ok(JxlDecoderStatus::NeedMoreInput) => {
                return Err(JxlError::NotEnoughInput);
            }
            Ok(JxlDecoderStatus::Success) => println!("Decoding successful"),
            Ok(status) => println!("Processing: {:?}", status),
            Err(e) => return Err(e),
        }
    }

    let basic_info = basic_info.expect("Basic info not received");

    // 원본 비트 깊이용 JxlPixelFormat 설정
    let original_format = create_pixel_format_from_basic_info(&basic_info);

    println!("Original format: {:?}", original_format);

    // 원본 비트 깊이로 디코딩
    let original_bit_depth = JxlBitDepth {
        type_: JxlBitDepthType::Custom,
        bits_per_sample: basic_info.bits_per_sample,
        exponent_bits_per_sample: basic_info.exponent_bits_per_sample,
    };

    let original_buffer = decode_buffer(&decoder, &original_format)?;

    // 결과 검증 (원본 비트 깊이)
    validate_decoded_data(&original_buffer, &original_format, &original_bit_depth);

    // 8비트 출력용 JxlPixelFormat 설정
    let bit8_format = create_pixel_format_from_basic_info(&basic_info);

    // 8비트로 디코딩
    let bit8_depth = JxlBitDepth {
        type_: JxlBitDepthType::Custom,
        bits_per_sample: 8,
        exponent_bits_per_sample: 0,
    };

    decoder.rewind(); // 디코더 상태 초기화
    decoder.set_input(&jxl_data).expect("Failed to set input"); // 입력 다시 설정

    let bit8_buffer = decode_buffer(&decoder, &bit8_format)?;

    // 8비트 결과 검증
    validate_decoded_data(&bit8_buffer, &bit8_format, &bit8_depth);

    Ok(())
}

fn validate_decoded_data(buffer: &[u8], format: &JxlPixelFormat, bit_depth: &JxlBitDepth) {
    let max_value = (1u32 << bit_depth.bits_per_sample) - 1;

    match format.data_type {
        JxlDataType::Uint8 => {
            assert!(
                buffer.iter().all(|&p| p as u32 <= max_value),
                "Pixel values exceed {} bits range",
                bit_depth.bits_per_sample
            );
            println!(
                "Successfully decoded to {} bit depth",
                bit_depth.bits_per_sample
            );
        }
        JxlDataType::Uint16 => {
            let pixels = buffer
                .chunks_exact(2)
                .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
                .collect::<Vec<u16>>();
            assert!(
                pixels.iter().all(|&p| p as u32 <= max_value),
                "Pixel values exceed {} bits range",
                bit_depth.bits_per_sample
            );
            println!(
                "Successfully decoded to {} bit depth",
                bit_depth.bits_per_sample
            );
        }
        JxlDataType::Float => {
            let pixels = buffer
                .chunks_exact(4)
                .map(|chunk| f32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect::<Vec<f32>>();
            assert!(
                pixels.iter().all(|&p| p.is_finite()),
                "Float pixel values are not finite"
            );
            println!("Successfully decoded float image");
        }
        _ => panic!("Unsupported data type"),
    }
}

fn create_pixel_format_from_basic_info(info: &JxlBasicInfo) -> JxlPixelFormat {
    let data_type = match info.bits_per_sample {
        0..=8 => JxlDataType::Uint8,
        9..=16 => JxlDataType::Uint16,
        _ => JxlDataType::Float, // 16비트 초과시 float 사용
    };

    let num_channels = if info.alpha_bits > 0 {
        info.num_color_channels + 1 // 알파 채널 포함
    } else {
        info.num_color_channels
    };

    let jsxlformat = JxlPixelFormat {
        num_channels,
        data_type,
        endianness: JxlEndianness::NativeEndian,
        align: 0,
    };
    println!("jsxlformat: {:?}", jsxlformat);
    jsxlformat
}

fn decode_buffer(decoder: &JxlDecoder, format: &JxlPixelFormat) -> Result<Vec<u8>, JxlError> {
    let mut buffer_size = 0;
    let mut buffer = Vec::new();

    loop {
        match decoder.process_input() {
            Ok(JxlDecoderStatus::Frame) => {
                // JXL_DEC_FRAME 이벤트 발생 시 버퍼 크기 계산
                buffer_size = match decoder.image_out_buffer_size(format) {
                    Ok(size) => {
                        println!("Buffer size calculated: {}", size);
                        size
                    }
                    Err(e) => {
                        println!("Failed to get image out buffer size: {:?}", e);
                        return Err(e);
                    }
                };
                buffer = vec![0u8; buffer_size];
                decoder.set_image_out_buffer(format, &mut buffer)?;
            }
            Ok(JxlDecoderStatus::NeedImageOutBuffer) => {
                if buffer.is_empty() {
                    return Err(JxlError::OutputBufferError);
                }
                decoder.set_image_out_buffer(format, &mut buffer)?;
            }
            Ok(JxlDecoderStatus::Success) => break,
            Ok(JxlDecoderStatus::NeedMoreInput) => return Err(JxlError::NotEnoughInput),
            Ok(status) => {
                println!("Processing: {:?}", status);
                continue;
            }
            Err(e) => {
                println!("Error during processing: {:?}", e);
                return Err(e);
            }
        }
    }

    Ok(buffer)
}

fn read_jxl_file(file_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
