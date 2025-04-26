use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct MenuResponse {
    pub message: String,
}

pub async fn bot_menu() -> Json<MenuResponse> {
    Json(MenuResponse {
        message: "*Fitur Bot :*

        command : /stiker
  1️⃣   Membuat Stiker dari Teks.
  2️⃣   Membuat Stiker dari Gambar.
  3️⃣   Membuat Stiker dari Teks dan Gambar.

🔧 Silakan pilih salah satu opsi di atas."
            .to_string(),
    })
}
