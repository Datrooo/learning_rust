mod hls;
mod loader_rustfs;
mod storage;
mod stream;
mod upload;
mod validation;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

struct ApiDoc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Using RustFS storage backend");
    let cfg = loader_rustfs::Config::from_env().expect("RustFS config: set RUSTFS_* env variables");
    let client = loader_rustfs::create_client(&cfg)
        .await
        .expect("Failed to create RustFS client");
    let storage = Arc::new(client);

    // в будущем сделать ограничения для доступа
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        //.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/media/upload", post(upload::upload_audio))
        .route("/hls/*path", get(stream::stream_hls))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .layer(cors)
        .with_state(storage);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to port 8080");

    tracing::info!("Backend listening on 0.0.0.0:8080");
    axum::serve(listener, app).await.expect("Server error");
}
