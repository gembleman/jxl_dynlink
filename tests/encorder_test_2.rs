use std::path::PathBuf;

fn get_test_dll_path() -> Result<PathBuf, &'static str> {
    let mut path = std::env::current_dir().unwrap();
    path.push("dlls");
    path.push("jxl.dll");

    if path.exists() {
        Ok(path)
    } else {
        Err("jxl.dll not found")
    }
}

#[test]
fn test_encode_jpg() {
    use jxl_dynlink::jpg_to_lossless_jxl;

    let path = get_test_dll_path();
    if path.is_err() {
        println!("{}", path.unwrap_err());
        return;
    }

    println!("dll path: {:?}", path.clone().unwrap());

    let input = "test_imgs/test_encode.jpg";
    let output = "test_imgs/test_encode.jxl";
    let effort: i64 = 9;
    let dll_path: PathBuf = path.unwrap();

    let input_data = std::fs::read(input).unwrap();

    let save_data = jpg_to_lossless_jxl(&dll_path, &input_data, effort).unwrap();

    std::fs::write(output, &save_data).unwrap();
}

#[test]
fn test_encode_png() {
    use jxl_dynlink::png_to_lossless_jxl;

    let path = get_test_dll_path();
    if path.is_err() {
        println!("{}", path.unwrap_err());
        return;
    }

    let input = "test_imgs/test_encode2.png";
    let output = "test_imgs/test_encode2.jxl";
    let effort: i64 = 9;
    let distance: f32 = 0.0;
    let dll_path: PathBuf = path.unwrap();

    let input_data = std::fs::read(input).unwrap();

    let save_data = png_to_lossless_jxl(&dll_path, &input_data, effort, distance).unwrap();

    std::fs::write(output, &save_data).unwrap();
}
