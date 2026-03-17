use anyhow::{Context, Result};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tracing::{error, info, warn};

use super::producer::PodcastEvent;
use crate::storage::StorageBackend;

const HLS_BUCKET: &str = "audio-hls";
const TOPIC: &str = "podcast";
const GROUP_ID: &str = "audio-upload-service";

pub async fn run_podcast_consumer(
    brokers: &str,
    storage: Arc<dyn StorageBackend>,
) -> Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", GROUP_ID)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "latest")
        .create()
        .context("Failed to create Kafka consumer")?;

    consumer
        .subscribe(&[TOPIC])
        .context("Failed to subscribe to podcast topic")?;

    info!("Kafka consumer started: listening on '{}'", TOPIC);

    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => {
                let payload = match msg.payload_view::<str>() {
                    Some(Ok(text)) => text,
                    Some(Err(e)) => {
                        warn!("Error decoding Kafka message payload: {}", e);
                        continue;
                    }
                    None => {
                        warn!("Empty Kafka message on {}", TOPIC);
                        continue;
                    }
                };

                info!("Received event on '{}': {}", TOPIC, payload);

                let event: PodcastEvent = match serde_json::from_str(payload) {
                    Ok(e) => e,
                    Err(e) => {
                        warn!("Failed to parse podcast event: {}", e);
                        continue;
                    }
                };

                match event.event_type.as_str() {
                    "deleted" => handle_deleted(&storage, &event).await,
                    "created" => {
                        info!(
                            "Podcast created: podcast_id={}, title={:?}",
                            event.podcast_id,
                            event.title
                        );
                        // добавить логику при создании подкаста, если вообще надо
                    }
                    "updated" => {
                        info!(
                            "Podcast updated: podcast_id={}, title={:?}",
                            event.podcast_id,
                            event.title
                        );
                        // добавить логику при обновлении подкаста, если вообще надо
                    }
                    other => {
                        warn!(
                            "Unknown event_type '{}' for podcast_id={}",
                            other, event.podcast_id
                        );
                    }
                }
            }
            Err(e) => {
                error!("Kafka consumer error: {}", e);
            }
        }
    }

    warn!("Kafka consumer stream ended unexpectedly");
    Ok(())
}

async fn handle_deleted(storage: &Arc<dyn StorageBackend>, event: &PodcastEvent) {
    let hls_path = match &event.hls_path {
        Some(path) => path,
        None => {
            warn!(
                "podcast.deleted event without hls_path, podcast_id={}",
                event.podcast_id
            );
            return;
        }
    };

    info!(
        "Processing podcast deleted: podcast_id={}, hls_path={}",
        event.podcast_id, hls_path
    );

    let prefix = extract_prefix(hls_path);

    if let Err(e) = delete_hls_objects(storage, &prefix).await {
        error!(
            "Failed to delete HLS objects for podcast_id={}: {}",
            event.podcast_id, e
        );
    } else {
        info!(
            "Deleted HLS objects for podcast_id={} (prefix={})",
            event.podcast_id, prefix
        );
    }
}

// извлекает префикс из пути
/// "audio-hls/stem/uuid/playlist.m3u8" → "stem/uuid"
fn extract_prefix(hls_path: &str) -> String {
    let without_bucket = hls_path
        .strip_prefix("audio-hls/")
        .unwrap_or(hls_path);
    match without_bucket.rsplit_once('/') {
        Some((prefix, _filename)) => prefix.to_string(),
        None => without_bucket.to_string(),
    }
}

// удаляет все hls объекты с таким префиксом
async fn delete_hls_objects(
    storage: &Arc<dyn StorageBackend>,
    prefix: &str,
) -> Result<()> {
    // playlist.m3u8, init.mp4, и сегменты seg_XXXXX.m4s
    let playlist_key = format!("{}/playlist.m3u8", prefix);
    if let Err(e) = storage.delete_object(HLS_BUCKET, &playlist_key).await {
        warn!("Failed to delete {}: {}", playlist_key, e);
    }

    let init_key = format!("{}/init.mp4", prefix);
    if let Err(e) = storage.delete_object(HLS_BUCKET, &init_key).await {
        tracing::debug!("init.mp4 not found or failed to delete: {}", e);
    }

    let mut i = 0;
    loop {
        let seg_key = format!("{}/seg_{:05}.m4s", prefix, i);
        match storage.delete_object(HLS_BUCKET, &seg_key).await {
            Ok(_) => {}
            Err(_) => {
                break;
            }
        }
        i += 1;
    }

    info!("Cleaned up HLS objects with prefix: {}", prefix);
    Ok(())
}
