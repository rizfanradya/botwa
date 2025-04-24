use base64::Engine as _;
use image::{GenericImageView, ImageFormat, Pixel, RgbaImage};
use infer;
use std::fs;
use std::process::Command;

pub async fn handle_sticker_image(image_data: &[u8]) -> String {
    // Mendeteksi format gambar dengan infer
    match infer::get(image_data) {
        Some(kind) => {
            let mime_type = kind.mime_type();
            println!("📦 Format gambar terdeteksi: {}", mime_type);

            // Proses gambar sesuai dengan format
            let img = match mime_type {
                "image/png" => image::load_from_memory_with_format(image_data, ImageFormat::Png),
                "image/jpeg" => image::load_from_memory_with_format(image_data, ImageFormat::Jpeg),
                "image/webp" => image::load_from_memory_with_format(image_data, ImageFormat::WebP),
                _ => {
                    eprintln!("⚠️ Format gambar tidak dikenali atau tidak didukung.");
                    return "".into();
                }
            };

            // Cek apakah proses load berhasil
            let img = match img {
                Ok(img) => img,
                Err(e) => {
                    eprintln!("❌ Gagal load image: {}", e);
                    return "".into();
                }
            };

            // Validasi ukuran gambar
            let (width, height) = img.dimensions();
            println!("📏 Ukuran gambar asli: {}x{}", width, height);
            if width == 0 || height == 0 {
                eprintln!("⚠️ Gambar kosong.");
                return "".into();
            }

            // Resize gambar ke 512x512
            let resized = img.resize(512, 512, image::imageops::FilterType::Lanczos3);

            // Salin ke RgbaImage
            let mut output = RgbaImage::new(512, 512);
            for (x, y, pixel) in resized.pixels() {
                output.put_pixel(x, y, pixel.to_rgba());
            }

            // Simpan sementara sebagai PNG
            let tmp_png = "tmp/output.png";
            let tmp_webp = "tmp/output.webp";
            if let Err(e) = output.save(tmp_png) {
                eprintln!("❌ Gagal simpan PNG: {}", e);
                return "".into();
            }

            // Jalankan `cwebp` untuk konversi ke WebP (jika memang perlu mengonversi)
            let status = Command::new("cwebp")
                .args([tmp_png, "-o", tmp_webp])
                .status();

            match status {
                Ok(s) if s.success() => {
                    // Baca file WebP hasil konversi
                    match fs::read(tmp_webp) {
                        Ok(webp_data) => {
                            // Encode base64
                            base64::engine::general_purpose::STANDARD.encode(webp_data)
                        }
                        Err(e) => {
                            eprintln!("❌ Gagal baca file WebP: {}", e);
                            "".into()
                        }
                    }
                }
                Ok(s) => {
                    eprintln!("❌ cwebp gagal dengan kode: {}", s);
                    "".into()
                }
                Err(e) => {
                    eprintln!("❌ Gagal jalankan cwebp: {}", e);
                    "".into()
                }
            }
        }
        None => {
            eprintln!("⚠️ Tidak dapat mendeteksi format gambar.");
            "".into()
        }
    }
}
