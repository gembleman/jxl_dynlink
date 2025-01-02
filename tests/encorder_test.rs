use image::{
    codecs::png::PngDecoder, ColorType, DynamicImage, GenericImageView, ImageDecoder, ImageFormat,
    ImageReader,
};
use jxl_dynlink::*;
use std::{
    fs::File,
    io::{BufReader, Read, Write},
};
const DLL_PATH: &str = "jxl.dll";

#[test]
fn jpg_to_lossless_jxl() -> Result<(), JxlError> {
    let input_path = "test_imgs/test_encode.jpg";
    let output_path = "test_imgs/test_encode.jxl";

    // Load the image using the image crate
    // let img = image::open(input_path).expect("Failed to load image");
    let dll_path = std::path::PathBuf::from(DLL_PATH);
    // Create a new JxlEncoder
    let mut encoder = JxlEncoder::new(&dll_path, None)?;

    // // Create frame settings
    let frame_settings: *mut std::ffi::c_void = encoder.create_frame_settings(None)?;
    println!("Frame settings created_1");

    println!("frame_settings: {:?}", frame_settings);
    encoder.set_frame_option(frame_settings, JxlEncoderFrameSettingId::Effort, 9)?;
    println!("Effort set");

    let file = File::open(input_path).map_err(|e| JxlError::InvalidInput(e.to_string()))?;
    let buffer_file = std::io::BufReader::new(file);
    let pixels = buffer_file.bytes().map(|b| b.unwrap()).collect::<Vec<u8>>();

    // Add jpg frame
    encoder.add_jpeg_frame(frame_settings, &pixels)?;
    println!("Image frame added");

    // Close input
    encoder.close_input()?;
    println!("Input closed");

    // Process output
    let mut compressed = Vec::new();
    let mut buffer = vec![0u8; pixels.len()];
    let mut next_out = buffer.as_mut_ptr();
    let mut avail_out = buffer.len();

    loop {
        let status = encoder.process_output(&mut next_out, &mut avail_out)?;
        println!("Status: {:?}", status);
        let bytes_written = buffer.len() - avail_out;
        compressed.extend_from_slice(&buffer[..bytes_written]);

        if status == JxlEncoderStatus::Success {
            break;
        } else if status == JxlEncoderStatus::NeedMoreOutput {
            next_out = buffer.as_mut_ptr();
            avail_out = buffer.len();
        } else {
            return Err(JxlError::EncodingFailed(status));
        }
    }

    // Write the compressed data to file
    let mut file = File::create(output_path).map_err(|e| JxlError::InvalidInput(e.to_string()))?;
    file.write_all(&compressed)
        .map_err(|e| JxlError::InvalidInput(e.to_string()))?;

    Ok(())
}

#[test]
fn png_to_lossless_jxl() -> Result<(), JxlError> {
    let input_path = "test_imgs/test_encode2.png";
    let output_path = "test_imgs/test_encode2.jxl";

    // Open the file once
    let file = File::open(input_path).expect("Failed to open file");
    let mut reader = ImageReader::new(BufReader::new(file));
    reader.set_format(ImageFormat::Png); // Specify the format if known

    let file_de = File::open(input_path).expect("Failed to open file");
    let reader_de = BufReader::new(file_de);
    // Create PNG decoder
    let mut decoder = PngDecoder::new(reader_de).expect("Failed to create PNG decoder");

    // Load the image
    let img = reader
        .decode()
        .map_err(|e| JxlError::InvalidInput(e.to_string()))?;
    let (width, height) = img.dimensions();

    // Create a new JxlEncoder
    let dll_path = std::path::PathBuf::from(DLL_PATH);
    let mut encoder = JxlEncoder::new(&dll_path, None)?;

    // Set up basic info
    let mut basic_info = JxlBasicInfo::default();
    encoder.init_basic_info(&mut basic_info)?;
    basic_info.xsize = width;
    basic_info.ysize = height;

    // Determine color channels, bit depth, and alpha based on image color type
    let (num_color_channels, bits_per_sample, alpha_bits) = match img.color() {
        ColorType::L8 => (1, 8, 0),
        ColorType::La8 => (1, 8, 8),
        ColorType::Rgb8 => (3, 8, 0),
        ColorType::Rgba8 => (3, 8, 8),
        ColorType::L16 => (1, 16, 0),
        ColorType::La16 => (1, 16, 16),
        ColorType::Rgb16 => (3, 16, 0),
        ColorType::Rgba16 => (3, 16, 16),
        _ => {
            return Err(JxlError::UnsupportedOperation(
                "Unsupported color type".to_string(),
            ))
        }
    };

    basic_info.num_color_channels = num_color_channels;
    basic_info.bits_per_sample = bits_per_sample;
    basic_info.alpha_bits = alpha_bits;
    basic_info.num_extra_channels = if alpha_bits > 0 { 1 } else { 0 };

    println!("basic_info: {:?}", basic_info);

    encoder.set_basic_info(&basic_info)?;
    println!("Basic info set");

    // Set color encoding
    if let Some(icc_profile) = decoder.icc_profile().expect("Failed to get ICC profile") {
        encoder.set_icc_profile(&icc_profile)?;
        println!("ICC profile set");
    } else {
        let mut color_encoding = JxlColorEncoding::default();
        match img.color() {
            ColorType::L8 | ColorType::La8 | ColorType::L16 | ColorType::La16 => {
                encoder.color_encoding_set_to_srgb(&mut color_encoding, true)?;
            }
            ColorType::Rgb8 | ColorType::Rgba8 | ColorType::Rgb16 | ColorType::Rgba16 => {
                encoder.color_encoding_set_to_srgb(&mut color_encoding, false)?;
            }
            _ => {
                return Err(JxlError::UnsupportedOperation(
                    "Unsupported color type for color encoding".to_string(),
                ));
            }
        }
        encoder.set_color_encoding(&color_encoding)?;
        println!("Color encoding set to sRGB");
    }

    // Create frame settings
    let frame_settings = encoder.create_frame_settings(None)?;
    println!("Frame settings created");

    // Set frame options
    encoder.set_frame_option(frame_settings, JxlEncoderFrameSettingId::Effort, 7)?;
    println!("Effort set");

    encoder.set_frame_distance(frame_settings, 0.0)?;
    println!("Distance set");

    let bit_depth = JxlBitDepth::default();

    encoder.set_frame_bit_depth(frame_settings, &bit_depth)?;
    println!("Bit depth set");

    // error.
    // encoder.set_frame_lossless(frame_settings, true)?;
    // println!("Lossless set");

    // Prepare pixel format based on the input image
    let (pixel_format, pixels) = match img {
        DynamicImage::ImageLuma8(_) => (
            JxlPixelFormat {
                num_channels: 1,
                data_type: JxlDataType::Uint8,
                endianness: JxlEndianness::NativeEndian,
                align: 0,
            },
            img.to_luma8().into_raw(),
        ),
        DynamicImage::ImageLumaA8(_) => (
            JxlPixelFormat {
                num_channels: 2,
                data_type: JxlDataType::Uint8,
                endianness: JxlEndianness::NativeEndian,
                align: 0,
            },
            img.to_luma_alpha8().into_raw(),
        ),
        DynamicImage::ImageRgb8(_) => (
            JxlPixelFormat {
                num_channels: 3,
                data_type: JxlDataType::Uint8,
                endianness: JxlEndianness::NativeEndian,
                align: 0,
            },
            img.to_rgb8().into_raw(),
        ),
        DynamicImage::ImageRgba8(_) => (
            JxlPixelFormat {
                num_channels: 4,
                data_type: JxlDataType::Uint8,
                endianness: JxlEndianness::NativeEndian,
                align: 0,
            },
            img.to_rgba8().into_raw(),
        ),
        DynamicImage::ImageLuma16(_) => {
            let raw_data = img.to_luma16().into_raw();
            let byte_data: Vec<u8> = raw_data
                .iter()
                .flat_map(|&pixel| pixel.to_ne_bytes())
                .collect();
            (
                JxlPixelFormat {
                    num_channels: 1,
                    data_type: JxlDataType::Uint16,
                    endianness: JxlEndianness::NativeEndian,
                    align: 0,
                },
                byte_data,
            )
        }
        DynamicImage::ImageLumaA16(_) => {
            let raw_data = img.to_luma16().into_raw();
            let byte_data: Vec<u8> = raw_data
                .iter()
                .flat_map(|&pixel| pixel.to_ne_bytes())
                .collect();
            (
                JxlPixelFormat {
                    num_channels: 2,
                    data_type: JxlDataType::Uint16,
                    endianness: JxlEndianness::NativeEndian,
                    align: 0,
                },
                byte_data,
            )
        }
        DynamicImage::ImageRgb16(_) => {
            let raw_data = img.to_luma16().into_raw();
            let byte_data: Vec<u8> = raw_data
                .iter()
                .flat_map(|&pixel| pixel.to_ne_bytes())
                .collect();

            (
                JxlPixelFormat {
                    num_channels: 3,
                    data_type: JxlDataType::Uint16,
                    endianness: JxlEndianness::NativeEndian,
                    align: 0,
                },
                byte_data,
            )
        }
        DynamicImage::ImageRgba16(_) => {
            let raw_data = img.to_luma16().into_raw();
            let byte_data: Vec<u8> = raw_data
                .iter()
                .flat_map(|&pixel| pixel.to_ne_bytes())
                .collect();

            (
                JxlPixelFormat {
                    num_channels: 4,
                    data_type: JxlDataType::Uint16,
                    endianness: JxlEndianness::NativeEndian,
                    align: 0,
                },
                byte_data,
            )
        }
        _ => {
            return Err(JxlError::UnsupportedOperation(
                "Unsupported image format".to_string(),
            ))
        }
    };

    // let file = File::open(input_path).map_err(|e| JxlError::InvalidInput(e.to_string()))?;
    // let buffer_file = std::io::BufReader::new(file);
    // let pixels = buffer_file.bytes().map(|b| b.unwrap()).collect::<Vec<u8>>();

    // Add frame
    encoder.add_image_frame(frame_settings, &pixel_format, &pixels)?;
    println!("Image frame added");

    // Close input
    encoder.close_input()?;
    println!("Input closed");

    // Process output
    let mut compressed = Vec::new();
    let mut buffer = vec![0u8; pixels.len()];
    let mut next_out = buffer.as_mut_ptr();
    let mut avail_out = buffer.len();

    loop {
        let status = encoder.process_output(&mut next_out, &mut avail_out)?;
        println!("Status: {:?}", status);
        let bytes_written = buffer.len() - avail_out;
        compressed.extend_from_slice(&buffer[..bytes_written]);

        if status == JxlEncoderStatus::Success {
            break;
        } else if status == JxlEncoderStatus::NeedMoreOutput {
            next_out = buffer.as_mut_ptr();
            avail_out = buffer.len();
        } else {
            return Err(JxlError::EncodingFailed(status));
        }
    }

    // Write the compressed data to file
    let mut file = File::create(output_path).expect("Failed to create file");
    file.write_all(&compressed)
        .expect("Failed to write to file");

    Ok(())
}
