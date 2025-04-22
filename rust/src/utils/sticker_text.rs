use ab_glyph::{FontArc, PxScale};
use base64::Engine as _;
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_text_mut, text_size};

pub async fn handle_sticker_text(text: &str, command: &str) -> String {
    let font = FontArc::try_from_vec(std::fs::read("fonts/Roboto-Bold.ttf").unwrap()).unwrap();
    let scale = PxScale::from(60.0);
    let mut image = RgbImage::from_pixel(512, 512, Rgb([255, 255, 255]));
    let line_height = 70;
    let lines: Vec<&str> = text
        .trim_start_matches(command)
        .trim()
        .split_whitespace()
        .collect();

    for (i, line) in lines.iter().enumerate() {
        let (w, _) = text_size(scale, &font, line);
        let x = (512 - w) / 2;
        let y = (512 / 2) - (line_height / 2) + i as u32 * line_height;
        draw_text_mut(
            &mut image,
            Rgb([0, 0, 0]),
            x as i32,
            y as i32,
            scale,
            &font,
            line,
        );
    }

    let tmp_path = "tmp/output.png";
    image.save(tmp_path).unwrap();

    std::process::Command::new("cwebp")
        .args([tmp_path, "-o", "tmp/output.webp"])
        .status()
        .unwrap();

    return base64::engine::general_purpose::STANDARD
        .encode(std::fs::read("tmp/output.webp").unwrap());
}
