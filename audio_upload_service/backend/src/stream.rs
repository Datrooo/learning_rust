use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};

use crate::upload::SharedStorage;

const HLS_BUCKET: &str = "audio-hls";

pub async fn stream_hls(
    State(storage): State<SharedStorage>,
    Path(object_key): Path<String>,
) -> impl IntoResponse {
    tracing::info!(
        "[stream] request: bucket={}, key={}",
        HLS_BUCKET,
        object_key
    );

    let data = match storage.get_object(HLS_BUCKET, &object_key).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::warn!("[stream] object not found: {}", e);
            return (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "text/plain".to_string())],
                format!("Object not found: {}", object_key).into_bytes(),
            );
        }
    };

    let content_type = guess_content_type(&object_key);

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type.to_string())],
        data,
    )
}

/// Определяем Content-Type по расширению файла
fn guess_content_type(key: &str) -> &'static str {
    if key.ends_with(".m3u8") {
        "application/vnd.apple.mpegurl"
    } else if key.ends_with(".m4s") {
        "video/iso.segment"
    } else if key.ends_with(".mp4") {
        "video/mp4"
    } else if key.ends_with(".ts") {
        "video/mp2t"
    } else {
        "application/octet-stream"
    }
}
