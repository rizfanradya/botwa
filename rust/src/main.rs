use axum::{
    Router,
    routing::{get, post},
    serve,
};
use tokio::net::TcpListener;

mod cruds;

#[tokio::main]
async fn main() {
    let api_routes = Router::new()
        .route("/menu", get(cruds::menu::bot_menu))
        .route("/txt_to_stk", post(cruds::txt_to_stk::text_to_sticker))
        .route("/img_to_stk", post(cruds::img_to_stk::image_to_sticker))
        .route(
            "/img_to_stk_with_txt",
            post(cruds::img_to_stk_with_txt::image_to_sticker_with_text),
        );

    let app = Router::new()
        .route("/", get(|| async { "Hello World" }))
        .nest("/api", api_routes);

    serve(TcpListener::bind("0.0.0.0:8001").await.unwrap(), app)
        .await
        .unwrap();
}
