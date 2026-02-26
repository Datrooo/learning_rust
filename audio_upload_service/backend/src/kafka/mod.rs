pub mod consumer;
pub mod producer;

use anyhow::{anyhow, Context, Result};
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::types::RDKafkaErrorCode;
use std::time::Duration;

pub use consumer::run_podcast_consumer;
pub use producer::{new_producer, SharedKafkaProducer};

const MEDIA_TOPIC: &str = "media";
const PODCAST_TOPIC: &str = "podcast";

pub async fn ensure_topics(brokers: &str) -> Result<()> {
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()
        .context("Failed to create Kafka admin client")?;

    let topics = [
        NewTopic::new(MEDIA_TOPIC, 1, TopicReplication::Fixed(1)),
        NewTopic::new(PODCAST_TOPIC, 1, TopicReplication::Fixed(1)),
    ];

    let results = admin
        .create_topics(
            &topics,
            &AdminOptions::new().operation_timeout(Some(Duration::from_secs(5))),
        )
        .await
        .context("Failed to create Kafka topics")?;

    for result in results {
        match result {
            Ok(topic) => tracing::info!("Kafka topic is ready: {}", topic),
            Err((topic, RDKafkaErrorCode::TopicAlreadyExists)) => {
                tracing::info!("Kafka topic already exists: {}", topic)
            }
            Err((topic, err)) => {
                return Err(anyhow!("Failed to ensure topic '{}': {:?}", topic, err));
            }
        }
    }

    Ok(())
}
