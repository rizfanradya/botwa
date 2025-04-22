use axum::{
    Router,
    routing::{get, post},
    serve,
};
use tokio::net::TcpListener;

mod cruds;
mod utils;

#[tokio::main]
async fn main() {
    serve(
        TcpListener::bind("0.0.0.0:8001").await.unwrap(),
        Router::new()
            .route("/", get(|| async { "Hello World" }))
            .route("/api/bot", post(cruds::bot::bot_handler)),
    )
    .await
    .unwrap();
}
