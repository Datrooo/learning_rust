use anyhow::{Context, Result};
use chrono::Utc;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaStartUploadEvent {
    pub event_type: String, // "media.start_upload"
    pub upload_id: String,
    pub filename: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUploadedEvent {
    pub event_type: String, // "media.uploaded"
    pub upload_id: String,
    pub file_id: String,
    pub filename: String,
    pub format: Option<String>,
    pub codec: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub duration_secs: Option<f64>,
    pub bit_rate: Option<u64>,
    pub size_bytes: usize,
    pub hls_path: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaErrorEvent {
    pub event_type: String, // "media.error"
    pub upload_id: String,
    pub error_message: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastEvent {
    pub event_type: String,
    pub podcast_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hls_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub timestamp: String,
}

pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(brokers: &str) -> Result<Self> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create::<FutureProducer>()
            .context("Failed to create Kafka producer")?;

        Ok(Self { producer })
    }

    pub async fn send_start_upload(
        &self,
        upload_id: Uuid,
        filename: &str,
    ) -> Result<()> {
        let event = MediaStartUploadEvent {
            event_type: "media.start_upload".to_string(),
            upload_id: upload_id.to_string(),
            filename: filename.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        };

        let payload = serde_json::to_string(&event)?;
        let record = FutureRecord::to("media")
            .key(&event.upload_id)
            .payload(&payload);

        self.producer
            .send(record, Duration::from_secs(30))
            .await
            .map_err(|(err, _msg)| anyhow::anyhow!("Failed to send media.start_upload: {}", err))?;

        tracing::info!(
            "Published event: media.start_upload (upload_id={})",
            upload_id
        );

        Ok(())
    }

    pub async fn send_uploaded(
        &self,
        upload_id: Uuid,
        filename: &str,
        format: Option<String>,
        codec: Option<String>,
        sample_rate: Option<u32>,
        channels: Option<u32>,
        duration_secs: Option<f64>,
        bit_rate: Option<u64>,
        size_bytes: usize,
        hls_path: &str,
    ) -> Result<()> {
        let event = MediaUploadedEvent {
            event_type: "media.uploaded".to_string(),
            upload_id: upload_id.to_string(),
            file_id: Uuid::new_v4().to_string(),
            filename: filename.to_string(),
            format,
            codec,
            sample_rate,
            channels,
            duration_secs,
            bit_rate,
            size_bytes,
            hls_path: hls_path.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        };

        let payload = serde_json::to_string(&event)?;
        let record = FutureRecord::to("media")
            .key(&event.file_id)
            .payload(&payload);

        self.producer
            .send(record, Duration::from_secs(30))
            .await
            .map_err(|(err, _msg)| anyhow::anyhow!("Failed to send media.uploaded: {}", err))?;

        tracing::info!(
            "Published event: media.uploaded (file_id={}, upload_id={})",
            event.file_id,
            upload_id
        );

        Ok(())
    }

    pub async fn send_error(
        &self,
        upload_id: Uuid,
        error_message: &str,
    ) -> Result<()> {
        let event = MediaErrorEvent {
            event_type: "media.error".to_string(),
            upload_id: upload_id.to_string(),
            error_message: error_message.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        };

        let payload = serde_json::to_string(&event)?;
        let record = FutureRecord::to("media")
            .key(&event.upload_id)
            .payload(&payload);

        self.producer
            .send(record, Duration::from_secs(30))
            .await
            .map_err(|(err, _msg)| anyhow::anyhow!("Failed to send media.error: {}", err))?;

        tracing::info!(
            "Published event: media.error (upload_id={})",
            upload_id
        );

        Ok(())
    }

    pub fn flush(&self) -> Result<()> {
        self.producer
            .flush(Duration::from_secs(10))
            .context("Failed to flush Kafka producer")?;
        Ok(())
    }
}

pub type SharedKafkaProducer = Arc<KafkaProducer>;

pub fn new_producer(brokers: &str) -> Result<SharedKafkaProducer> {
    let producer = KafkaProducer::new(brokers)?;
    Ok(Arc::new(producer))
}
