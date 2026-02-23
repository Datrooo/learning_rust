use anyhow::{Context, Result};
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_s3::error::ProvideErrorMetadata;
use aws_sdk_s3::{primitives::ByteStream, Client};
use std::{env, path::Path, path::PathBuf};
use tokio::fs;
use tracing::info;

pub struct Config {
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            region: env::var("RUSTFS_REGION")?,
            access_key_id: env::var("RUSTFS_ACCESS_KEY_ID")?,
            secret_access_key: env::var("RUSTFS_SECRET_ACCESS_KEY")?,
            endpoint_url: env::var("RUSTFS_ENDPOINT_URL")?,
        })
    }
}

pub struct RustFsClient {
    client: Client,
}

impl RustFsClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn new_bucket(&self, bucket: &str) -> Result<()> {
        match self.client.create_bucket().bucket(bucket).send().await {
            Ok(_) => info!("Bucket '{}' created successfully", bucket),
            Err(err) if err.code() == Some("BucketAlreadyOwnedByYou") => {
                info!("Bucket '{}' already exists, skip create", bucket);
            }
            Err(err) => {
                tracing::error!(
                    "create_bucket error: code={:?}, message={:?}, raw={:?}",
                    err.code(),
                    err.message(),
                    err
                );
                return Err(err).with_context(|| format!("create_bucket failed for {bucket}"));
            }
        }

        Ok(())
    }

    pub async fn del_bucket(&self, bucket_name: &str) -> Result<()> {
        info!("Deleting bucket '{}'", bucket_name);

        match self.client.delete_bucket().bucket(bucket_name).send().await {
            Ok(_) => info!("Bucket '{}' deleted successfully", bucket_name),
            Err(err) if err.code() == Some("NoSuchBucket") => {
                info!("Bucket '{}' does not exist, skip delete", bucket_name);
            }
            Err(err) => {
                return Err(err).with_context(|| format!("delete_bucket failed for {bucket_name}"));
            }
        }

        Ok(())
    }

    pub async fn list_buckets(&self) -> Result<()> {
        info!("starting printing all buckets");

        let res = self
            .client
            .list_buckets()
            .send()
            .await
            .with_context(|| "error listing buckets")?;

        println!("Total buckets number is {}", res.buckets().len());
        for bucket in res.buckets() {
            println!("Bucket: {:?}", bucket.name());
        }

        info!("Buckets listed successully");
        Ok(())
    }

    pub async fn list_objects(&self, bucket_name: &str) -> Result<()> {
        info!("starting printing all objects in bucket '{}'", bucket_name);

        let res = self
            .client
            .list_objects_v2()
            .bucket(bucket_name)
            .send()
            .await
            .with_context(|| format!("error listing objects in {bucket_name}"))?;

        println!("Total objects number is {}", res.contents().len());
        for object in res.contents() {
            println!("Object: {:?}", object.key());
        }

        info!("objects listed successfully");
        Ok(())
    }

    pub async fn upload_file(
        &self,
        filepath: &Path,
        filename: &str,
        bucket_name: &str,
    ) -> Result<()> {
        info!("started to upload file");

        if !filepath.exists() {
            anyhow::bail!("file does not exist: {}", filepath.display());
        }

        let data = fs::read(filepath)
            .await
            .with_context(|| format!("can not open file {}", filepath.display()))?;
        let size_bytes = data.len();

        self.client
            .put_object()
            .bucket(bucket_name)
            .key(filename)
            .body(ByteStream::from(data))
            .send()
            .await
            .with_context(|| format!("error uploading '{filename}' to bucket '{bucket_name}'"))?;

        info!(
            "uploaded successfully: bucket='{}', object='{}', bytes={}",
            bucket_name, filename, size_bytes
        );
        Ok(())
    }

    /// Скачивает объект из бакета и возвращает сырые байты
    pub async fn get_bytes(&self, bucket_name: &str, object_key: &str) -> Result<Vec<u8>> {
        let res = self
            .client
            .get_object()
            .bucket(bucket_name)
            .key(object_key)
            .send()
            .await
            .with_context(|| {
                format!("error downloading object '{object_key}' from '{bucket_name}'")
            })?;

        let bytes = res
            .body
            .collect()
            .await
            .with_context(|| format!("error reading object body '{object_key}'"))?
            .into_bytes();

        Ok(bytes.to_vec())
    }

    pub async fn download_file(&self, bucket_name: &str, filename: &str) -> Result<()> {
        info!("started downloading");

        let bytes = self.get_bytes(bucket_name, filename).await?;

        let output_path: PathBuf = Path::new("/tmp/downloads").join(filename);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("error creating dir {}", parent.display()))?;
        }
        fs::write(&output_path, &bytes).await.with_context(|| {
            format!("error writing downloaded file to {}", output_path.display())
        })?;

        info!(
            "downloaded successfully: object='{}', bytes={}, path='{}'",
            filename,
            bytes.len(),
            output_path.display()
        );
        Ok(())
    }
}

pub async fn create_client(cfg: &Config) -> Result<RustFsClient> {
    let credentials = Credentials::new(
        cfg.access_key_id.clone(),
        cfg.secret_access_key.clone(),
        None,
        None,
        "rustfs",
    );

    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(cfg.region.clone()))
        .credentials_provider(credentials)
        .endpoint_url(cfg.endpoint_url.clone())
        .load()
        .await;

    let s3_config = aws_sdk_s3::config::Builder::from(&shared_config)
        .force_path_style(true)
        .build();

    Ok(RustFsClient::new(Client::from_conf(s3_config)))
}


use crate::storage::StorageBackend;
use async_trait::async_trait;

#[async_trait]
impl StorageBackend for RustFsClient {
    fn name(&self) -> &str {
        "rustfs"
    }

    async fn ensure_bucket(&self, bucket: &str) -> Result<()> {
        self.new_bucket(bucket).await
    }

    async fn upload_file(
        &self,
        local_path: &Path,
        bucket: &str,
        object_key: &str,
    ) -> Result<()> {
        RustFsClient::upload_file(self, local_path, object_key, bucket).await
    }

    async fn get_object(
        &self,
        bucket: &str,
        object_key: &str,
    ) -> Result<Vec<u8>> {
        self.get_bytes(bucket, object_key).await
    }
}
