use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
};
use dashmap::DashMap;
use serde::Serialize;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use utoipa::ToSchema;

use crate::upload::AppState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Receiving,
    Validating,
    Converting,
    Uploading,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UploadProgress {
    pub stage: Stage,
    pub bytes_received: usize,
    pub total_expected: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl UploadProgress {
    pub fn new() -> Self {
        Self {
            stage: Stage::Receiving,
            bytes_received: 0,
            total_expected: None,
            message: None,
        }
    }
}

pub type ProgressMap = Arc<DashMap<Uuid, UploadProgress>>;

pub fn new_progress_map() -> ProgressMap {
    Arc::new(DashMap::new())
}

type SseStream = Pin<Box<dyn futures_core::Stream<Item = Result<Event, Infallible>> + Send>>;

#[utoipa::path(
    get,
    path = "/api/media/progress/{upload_id}",
    params(
        ("upload_id" = Uuid, Path, description = "Upload id")
    ),
    responses(
        (status = 200, description = "SSE progress stream", content_type = "text/event-stream", body = UploadProgress)
    ),
    tag = "media"
)]
pub async fn progress_sse(
    State(state): State<AppState>,
    Path(upload_id): Path<Uuid>,
) -> Sse<SseStream> {
    let progress_map = state.progress.clone();

    let stream: SseStream = Box::pin(async_stream::stream! {
        let mut interval = tokio::time::interval(Duration::from_millis(300));

        let mut waited = 0u32;
        while !progress_map.contains_key(&upload_id) {
            if waited >= 100 {
                yield Ok(Event::default()
                    .event("error")
                    .data(r#"{"error":"upload not found"}"#));
                return;
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
            waited += 1;
        }

        loop {
            interval.tick().await;

            let entry = progress_map.get(&upload_id);

            match entry {
                Some(p) => {
                    let progress = p.clone();
                    drop(p);

                    let json = serde_json::to_string(&progress).unwrap_or_default();
                    yield Ok(Event::default().data(json));

                    if progress.stage == Stage::Done || progress.stage == Stage::Error {
                        break;
                    }
                }
                None => {
                    break;
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
        progress_map.remove(&upload_id);
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
