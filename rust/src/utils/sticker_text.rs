use ab_glyph::{FontArc, PxScale};
use base64::Engine as _;
use image::{Rgb, RgbImage, imageops};
use imageproc::drawing::draw_text_mut;
use imageproc::drawing::text_size;

pub async fn handle_sticker_text(text: &str, command: &str) -> String {
    let font = FontArc::try_from_vec(std::fs::read("fonts/Archivo-Regular.ttf").unwrap()).unwrap();
    let scale = PxScale::from(130.0);
    let mut image = RgbImage::from_pixel(512, 512, Rgb([255, 255, 255]));
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();
    let words: Vec<&str> = text
        .trim_start_matches(command)
        .trim()
        .split_whitespace()
        .collect();

    for word in words {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };
        let (line_width, _) = text_size(scale, &font, &test_line);
        if line_width > 492 as u32 {
            lines.push(current_line);
            current_line = word.to_string();
        } else {
            current_line = test_line;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    let mut y = 10;
    for line in lines.iter() {
        draw_text_mut(&mut image, Rgb([0, 0, 0]), 10, y as i32, scale, &font, line);
        y += 130;
    }

    image = imageops::blur(&image, 4.0);
    let tmp_path = "tmp/output.png";
    image.save(tmp_path).unwrap();

    std::process::Command::new("cwebp")
        .args([tmp_path, "-o", "tmp/output.webp"])
        .status()
        .unwrap();

    return base64::engine::general_purpose::STANDARD
        .encode(std::fs::read("tmp/output.webp").unwrap());
}
