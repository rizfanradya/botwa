use ab_glyph::{FontArc, PxScale};
use axum::response::Json;
use axum_extra::extract::multipart::{Multipart, MultipartError};
use base64::Engine as _;
use image::{GenericImageView, ImageFormat, Pixel, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use serde::Serialize;
use std::{fs, process::Command};

#[derive(Serialize)]
pub struct StickerResponse {
    pub buffer: String,
}

pub async fn image_to_sticker_with_text(
    mut multipart: Multipart,
) -> Result<Json<StickerResponse>, MultipartError> {
    let mut text = String::new();
    let mut image_data: Option<bytes::Bytes> = None;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "text" => {
                text = field.text().await.unwrap_or_default();
            }
            "file" => {
                image_data = Some(field.bytes().await?);
            }
            _ => {}
        }
    }

    let bytes = image_data.expect("❌ Gambar tidak ditemukan");
    let mime_type = infer::get(&bytes)
        .expect("⚠️ Format tidak terdeteksi")
        .mime_type();

    let img = match mime_type {
        "image/png" => image::load_from_memory_with_format(&bytes, ImageFormat::Png),
        "image/jpeg" => image::load_from_memory_with_format(&bytes, ImageFormat::Jpeg),
        "image/webp" => image::load_from_memory_with_format(&bytes, ImageFormat::WebP),
        _ => panic!("⚠️ Format gambar tidak didukung"),
    }
    .expect("❌ Gagal memuat gambar");

    let (width, height) = img.dimensions();
    let mut output = RgbaImage::new(width, height);
    for (x, y, pixel) in img.pixels() {
        output.put_pixel(x, y, pixel.to_rgba());
    }

    let font =
        FontArc::try_from_vec(std::fs::read("data/fonts/Archivo-Regular.ttf").unwrap()).unwrap();
    let mut best_scale = PxScale::from(130.0);
    let max_text_width = width - 40;
    let mut longest_line = text.clone();

    for word in text.split_whitespace() {
        if word.len() > longest_line.len() {
            longest_line = word.to_string();
        }
    }

    for size in (20..=130).rev() {
        let test_scale = PxScale::from(size as f32);
        let (line_width, _) = imageproc::drawing::text_size(test_scale, &font, &longest_line);
        if line_width <= max_text_width {
            best_scale = test_scale;
            break;
        }
    }

    let scale = best_scale;
    let mut lines = Vec::new();
    let mut current_line = String::new();
    for word in text.split_whitespace() {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };
        let (line_width, _) = imageproc::drawing::text_size(scale, &font, &test_line);
        if line_width > max_text_width {
            lines.push(current_line);
            current_line = word.to_string();
        } else {
            current_line = test_line;
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    let total_text_height: u32 = lines.len() as u32 * scale.y.ceil() as u32;
    let y_start = height - total_text_height - 10;
    let offsets = [
        (-1, 0),
        (1, 0),
        (0, -1),
        (0, 1),
        (-1, -1),
        (-1, 1),
        (1, -1),
        (1, 1),
    ];

    let mut y_offset = y_start;
    for line in lines.iter() {
        let (text_width, text_height) = imageproc::drawing::text_size(scale, &font, line);
        let x_offset = (width - text_width) / 2;

        for (dx, dy) in &offsets {
            draw_text_mut(
                &mut output,
                Rgba([0, 0, 0, 255]),
                x_offset as i32 + dx,
                y_offset as i32 + dy,
                scale,
                &font,
                line,
            );
        }

        draw_text_mut(
            &mut output,
            Rgba([255, 255, 255, 255]),
            x_offset as i32,
            y_offset as i32,
            scale,
            &font,
            line,
        );

        y_offset += text_height as u32;
    }

    let tmp_png = "data/tmp/output.png";
    let tmp_webp = "data/tmp/output.webp";
    output.save(tmp_png).expect("❌ Gagal simpan PNG");

    Command::new("cwebp")
        .args([tmp_png, "-o", tmp_webp])
        .status()
        .expect("❌ Gagal menjalankan cwebp");

    Ok(Json(StickerResponse {
        buffer: base64::engine::general_purpose::STANDARD
            .encode(fs::read(tmp_webp).expect("❌ Gagal baca file WebP")),
    }))
}
