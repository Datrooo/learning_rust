use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

use crate::hls::HlsOutput;

#[async_trait]
pub trait StorageBackend: Send + Sync {
    fn name(&self) -> &str;

    async fn ensure_bucket(&self, bucket: &str) -> Result<()>;

    async fn upload_file(&self, local_path: &Path, bucket: &str, object_key: &str) -> Result<()>;

    async fn get_object(&self, bucket: &str, object_key: &str) -> Result<Vec<u8>>;

    async fn upload_hls_output(&self, hls: &HlsOutput, bucket: &str, prefix: &str) -> Result<()> {
        let files = hls.list_files().await.map_err(|e| anyhow::anyhow!(e))?;

        tracing::info!(
            "[{}] Uploading {} HLS files to {}/{}",
            self.name(),
            files.len(),
            bucket,
            prefix
        );

        for file_path in &files {
            let file_name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let object_key = if prefix.is_empty() {
                file_name.to_string()
            } else {
                format!("{}/{}", prefix, file_name)
            };

            self.upload_file(file_path, bucket, &object_key).await?;
        }

        tracing::info!(
            "[{}] HLS upload complete: {}/{}",
            self.name(),
            bucket,
            prefix
        );

        Ok(())
    }

    async fn delete_object(&self, bucket: &str, object_key: &str) -> Result<()>;
}
