use ab_glyph::{FontArc, PxScale};
use axum::extract::Json;
use base64::Engine as _;
use image::{Rgb, RgbImage, imageops};
use imageproc::drawing::{draw_text_mut, text_size};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

#[derive(Deserialize)]
pub struct ReqBody {
    text: String,
}

#[derive(Serialize)]
pub struct StickerResponse {
    pub buffer: String,
}

pub async fn text_to_sticker(Json(msg): Json<ReqBody>) -> Json<StickerResponse> {
    let font = FontArc::try_from_vec(
        std::fs::read("data/fonts/Archivo-Regular.ttf").expect("❌ Gagal memuat font"),
    )
    .expect("❌ Font tidak valid");

    let mut scale = PxScale::from(130.0);
    let max_width = 492u32;
    let max_height = 492u32;

    let wrap_text = |scale: PxScale| {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        for word in msg.text.split_whitespace() {
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
    let tmp_path = "data/tmp/output.png";
    image.save(tmp_path).expect("❌ Gagal menyimpan gambar");

    let tmp_webp = "data/tmp/output.webp";
    Command::new("cwebp")
        .args([tmp_path, "-o", tmp_webp])
        .status()
        .expect("❌ Gagal menjalankan cwebp");

    Json(StickerResponse {
        buffer: base64::engine::general_purpose::STANDARD
            .encode(fs::read(tmp_webp).expect("❌ Gagal membaca file WebP")),
    })
}
