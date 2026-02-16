mod upload;
mod validation;
mod loader;

use axum::{extract::DefaultBodyLimit, routing::post, Router};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // в будущем сделать ограничения для доступа
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/upload", post(upload::upload_audio))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) 
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to port 8080");

    tracing::info!("Backend listening on 0.0.0.0:8080");
    axum::serve(listener, app).await.expect("Server error");
}
