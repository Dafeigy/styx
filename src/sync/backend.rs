use crate::sync::manifest::SyncManifest;
use anyhow::Context;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use std::path::Path;

/// Configuration for the S3 backend, resolved from environment variables.
pub struct S3Config {
    pub endpoint: String,
    pub bucket_name: String,
    pub prefix: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

impl S3Config {
    /// Build config from STYX_S3_* environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        let endpoint = std::env::var("STYX_S3_ENDPOINT")
            .unwrap_or_else(|_| "https://s3.amazonaws.com".to_string());

        let bucket_name = std::env::var("STYX_S3_BUCKET")
            .context("STYX_S3_BUCKET is required for sync operations")?;

        let prefix = std::env::var("STYX_S3_PREFIX")
            .unwrap_or_else(|_| "styx/".to_string());

        let region = std::env::var("STYX_S3_REGION")
            .unwrap_or_else(|_| "us-east-1".to_string());

        let access_key = std::env::var("STYX_S3_ACCESS_KEY")
            .context("STYX_S3_ACCESS_KEY is required for sync operations")?;

        let secret_key = std::env::var("STYX_S3_SECRET_KEY")
            .context("STYX_S3_SECRET_KEY is required for sync operations")?;

        Ok(Self { endpoint, bucket_name, prefix, region, access_key, secret_key })
    }
}

/// An S3-compatible storage backend.
pub struct S3Backend {
    bucket: Bucket,
    prefix: String,
}

impl S3Backend {
    /// Create a new S3 backend from environment config.
    pub fn from_env() -> anyhow::Result<Self> {
        let config = S3Config::from_env()?;

        let region = Region::Custom {
            region: config.region,
            endpoint: config.endpoint,
        };

        let credentials = Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| anyhow::anyhow!("S3 credentials error: {}", e))?;

        // Ensure prefix ends with /
        let prefix = if config.prefix.ends_with('/') {
            config.prefix
        } else {
            format!("{}/", config.prefix)
        };

        let bucket =
            Bucket::new(&config.bucket_name, region, credentials)
                .map_err(|e| anyhow::anyhow!("S3 bucket error: {}", e))?;

        Ok(Self { bucket, prefix })
    }

    /// Key for the manifest file in S3.
    fn manifest_key(&self) -> String {
        format!("{}manifest.json", self.prefix)
    }

    /// Key for a database file in S3.
    fn db_key(&self, name: &str) -> String {
        format!("{}{}.redb", self.prefix, name)
    }

    /// Fetch the remote sync manifest.
    pub async fn get_manifest(&self) -> anyhow::Result<Option<SyncManifest>> {
        let key = self.manifest_key();

        match self.bucket.get_object(&key).await {
            Ok(data) => {
                let bytes: Vec<u8> = data.to_vec();
                let manifest: SyncManifest = serde_json::from_slice(&bytes)
                    .with_context(|| "failed to parse remote manifest")?;
                Ok(Some(manifest))
            }
            Err(e) => {
                let err_str = format!("{}", e);
                if err_str.contains("404") || err_str.contains("NoSuchKey") {
                    // No manifest yet — first sync.
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!("S3 error fetching manifest: {}", e))
                }
            }
        }
    }

    /// Upload the sync manifest to S3.
    pub async fn put_manifest(&self, manifest: &SyncManifest) -> anyhow::Result<()> {
        let key = self.manifest_key();
        let data = serde_json::to_vec_pretty(manifest)?;

        self.bucket
            .put_object(&key, &data)
            .await
            .map_err(|e| anyhow::anyhow!("S3 error uploading manifest: {}", e))?;
        Ok(())
    }

    /// Download a database file from S3 to a local path.
    pub async fn download_db(&self, name: &str, dest: &Path) -> anyhow::Result<()> {
        let key = self.db_key(name);

        let data = self
            .bucket
            .get_object(&key)
            .await
            .map_err(|e| anyhow::anyhow!("S3 error downloading @{}: {}", name, e))?;
        let bytes: Vec<u8> = data.to_vec();

        std::fs::write(dest, &bytes)
            .with_context(|| format!("failed to write downloaded database to {}", dest.display()))?;

        Ok(())
    }

    /// Upload a local database file to S3.
    pub async fn upload_db(&self, name: &str, source: &Path) -> anyhow::Result<()> {
        let key = self.db_key(name);
        let data = std::fs::read(source)
            .with_context(|| format!("failed to read local database {}", source.display()))?;

        self.bucket
            .put_object(&key, &data)
            .await
            .map_err(|e| anyhow::anyhow!("S3 error uploading @{}: {}", name, e))?;

        Ok(())
    }
}
