mod api_doc;
mod hls;
mod kafka;
mod loader_rustfs;
mod progress;
mod storage;
mod stream;
mod upload;
mod validation;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use api_doc::ApiDoc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Using RustFS storage backend");
    let cfg = loader_rustfs::Config::from_env().expect("RustFS config: set RUSTFS_* env variables");
    let client = loader_rustfs::create_client(&cfg)
        .await
        .expect("Failed to create RustFS client");

    let kafka_brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let kafka_producer = kafka::new_producer(&kafka_brokers)
        .expect("Failed to create Kafka producer");

    let state = upload::AppState {
        storage: std::sync::Arc::new(client),
        progress: progress::new_progress_map(),
        kafka: kafka_producer,
    };

    let consumer_storage = state.storage.clone();
    let consumer_brokers = kafka_brokers.clone();
    tokio::spawn(async move {
        if let Err(e) = kafka::run_podcast_consumer(&consumer_brokers, consumer_storage).await {
            tracing::error!("Kafka consumer crashed: {}", e);
        }
    });

    // в будущем сделать ограничения для доступа
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/media/upload", post(upload::upload_audio))
        .route("/api/media/progress/:upload_id", get(progress::progress_sse))
        .route("/hls/*path", get(stream::stream_hls))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to port 8080");

    tracing::info!("Backend listening on http://0.0.0.0:8080");
    tracing::info!("Swagger UI: http://localhost:8080/swagger-ui/");
    tracing::info!("OpenAPI JSON: http://localhost:8080/api-docs/openapi.json");
    axum::serve(listener, app).await.expect("Server error");
}
