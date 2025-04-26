use axum::{body::Bytes, response::Json};
use base64::Engine as _;
use image::{GenericImageView, ImageFormat, Pixel, RgbaImage};
use infer;
use serde::Serialize;
use std::{fs, process::Command};

#[derive(Serialize)]
pub struct StickerResponse {
    pub buffer: String,
}

pub async fn image_to_sticker(bytes: Bytes) -> Json<StickerResponse> {
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

    let resized = img.resize(512, 512, image::imageops::FilterType::Lanczos3);
    let (new_w, new_h) = resized.dimensions();
    let mut output = RgbaImage::new(512, 512);
    let offset_x = (512 - new_w) / 2;
    let offset_y = (512 - new_h) / 2;

    for (x, y, pixel) in resized.pixels() {
        output.put_pixel(x + offset_x, y + offset_y, pixel.to_rgba());
    }

    let tmp_png = "data/tmp/output.png";
    let tmp_webp = "data/tmp/output.webp";
    output.save(tmp_png).expect("❌ Gagal simpan PNG");

    Command::new("cwebp")
        .args([tmp_png, "-o", tmp_webp])
        .status()
        .expect("❌ Gagal menjalankan cwebp");

    Json(StickerResponse {
        buffer: base64::engine::general_purpose::STANDARD
            .encode(fs::read(tmp_webp).expect("❌ Gagal baca file WebP")),
    })
}
