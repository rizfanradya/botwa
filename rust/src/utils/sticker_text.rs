use ab_glyph::{FontArc, PxScale};
use base64::Engine as _;
use image::{Rgb, RgbImage, imageops};
use imageproc::drawing::{draw_text_mut, text_size};

pub async fn handle_sticker_text(text: &str, command: &str) -> String {
    let font = FontArc::try_from_vec(std::fs::read("fonts/Archivo-Regular.ttf").unwrap()).unwrap();
    let raw_text = text.trim_start_matches(command).trim();
    let mut scale = PxScale::from(130.0);
    let max_width = 492u32;
    let max_height = 492u32;

    let wrap_text = |scale: PxScale| {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        for word in raw_text.split_whitespace() {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };
            let (line_width, _) = text_size(scale, &font, &test_line);
            if line_width > max_width {
                lines.push(current_line);
                current_line = word.to_string();
            } else {
                current_line = test_line;
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        lines
    };

    let mut lines;
    loop {
        lines = wrap_text(scale);
        let total_height = lines.len() as u32 * scale.y.ceil() as u32;
        if total_height <= max_height || scale.y < 20.0 {
            break;
        }
        scale = PxScale::from(scale.y - 2.0);
    }

    let mut image = RgbImage::from_pixel(512, 512, Rgb([255, 255, 255]));
    let mut y = (512 - (lines.len() as u32 * scale.y.ceil() as u32)) / 2;
    for line in lines.iter() {
        draw_text_mut(&mut image, Rgb([0, 0, 0]), 10, y as i32, scale, &font, line);
        y += scale.y.ceil() as u32;
    }

    image = imageops::blur(&image, 4.0);
    let tmp_path = "tmp/output.png";
    image.save(tmp_path).unwrap();

    std::process::Command::new("cwebp")
        .args([tmp_path, "-o", "tmp/output.webp"])
        .status()
        .unwrap();

    base64::engine::general_purpose::STANDARD.encode(std::fs::read("tmp/output.webp").unwrap())
}
