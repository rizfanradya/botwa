use axum::{extract::Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Deserialize)]
struct ExtendedTextMessage {
    text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageMessage {
    url: Option<String>,
    caption: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageContent {
    conversation: Option<String>,
    extended_text_message: Option<ExtendedTextMessage>,
    image_message: Option<ImageMessage>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageKey {
    remote_jid: String,
}

#[derive(Deserialize)]
pub struct WhatsAppMessage {
    key: MessageKey,
    message: Option<MessageContent>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum AxumResponse {
    #[serde(rename_all = "camelCase")]
    None { text: String },
    #[serde(rename_all = "camelCase")]
    Text { text: String },
    #[serde(rename_all = "camelCase")]
    Sticker { buffer: String },
}

pub async fn bot_handler(Json(msg): Json<WhatsAppMessage>) -> impl IntoResponse {
    let jid = &msg.key.remote_jid;
    let text_opt = msg.message.as_ref().and_then(|m| {
        m.conversation
            .clone()
            .or_else(|| m.extended_text_message.as_ref().map(|e| e.text.clone()))
            .or_else(|| m.image_message.as_ref().and_then(|img| img.caption.clone()))
    });

    if let Some(text) = text_opt {
        println!("Pesan masuk dari {}: {}", jid, text);

        if text.starts_with(".bts ") {
            return Json(AxumResponse::Sticker {
                buffer: utils::sticker_text::handle_sticker_text(&text, ".bts ").await,
            });
        } else if text.starts_with(".bfs") {
            if let Some(image_message) = msg.message.as_ref().and_then(|m| m.image_message.as_ref())
            {
                if let Some(image_url) = &image_message.url {
                    match reqwest::get(image_url).await {
                        Ok(response) => {
                            let content_type = response
                                .headers()
                                .get("Content-Type")
                                .map(|v| v.to_str().unwrap_or("-"))
                                .unwrap_or("-")
                                .to_string();
                            let bytes = response.bytes().await.unwrap();
                            println!("📦 Content-Type: {}", content_type);
                            println!("📏 Size: {} bytes", bytes.len());
                            std::fs::write("tmp/raw_blob.bin", &bytes).unwrap();
                            let sticker = utils::img_to_stk::handle_sticker_image(&bytes).await;
                            return Json(AxumResponse::Sticker { buffer: sticker });
                        }
                        Err(_) => {
                            return Json(AxumResponse::None {
                                text: "⚠️ Gagal fetch gambar.".into(),
                            });
                        }
                    }
                }
            }
        } else if text.starts_with(".bm") {
            return Json(AxumResponse::Text {
                text: utils::menu::bot_menu().await,
            });
        } else {
            return Json(AxumResponse::None {
                text: "⚠️ Perintah tidak dikenali.".into(),
            });
        }
    }

    Json(AxumResponse::None {
        text: "⚠️ Tidak ada isi pesan.".into(),
    })
}
