use axum::{extract::Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Deserialize)]
struct ExtendedTextMessage {
    text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageContent {
    conversation: Option<String>,
    extended_text_message: Option<ExtendedTextMessage>,
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
    });

    if let Some(text) = text_opt {
        println!("Pesan masuk dari {}: {}", jid, text);

        if text.starts_with(".bot text ") {
            return Json(AxumResponse::Sticker {
                buffer: utils::sticker_text::handle_sticker_text(&text, ".bot text ").await,
            });
        } else if text.starts_with(".bot") {
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
