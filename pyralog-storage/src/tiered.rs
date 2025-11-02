use pyralog_core::{LogOffset, Result, PyralogError};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Tiered storage for offloading cold data to object storage
/// Inspired by Redpanda's tiered storage feature
pub struct TieredStorage {
    local_path: PathBuf,
    remote_config: RemoteStorageConfig,
}

#[derive(Debug, Clone)]
pub enum RemoteStorageConfig {
    S3 {
        bucket: String,
        region: String,
        access_key: String,
        secret_key: String,
    },
    Azure {
        container: String,
        connection_string: String,
    },
    Gcs {
        bucket: String,
        credentials_path: PathBuf,
    },
    Local {
        path: PathBuf,
    },
}

impl TieredStorage {
    pub fn new(local_path: PathBuf, remote_config: RemoteStorageConfig) -> Self {
        Self {
            local_path,
            remote_config,
        }
    }

    /// Upload a segment to remote storage
    pub async fn upload_segment(&self, segment_path: &Path) -> Result<String> {
        match &self.remote_config {
            RemoteStorageConfig::Local { path } => {
                let filename = segment_path
                    .file_name()
                    .ok_or_else(|| PyralogError::StorageError("Invalid segment path".to_string()))?;
                
                let remote_path = path.join(filename);
                
                fs::copy(segment_path, &remote_path)
                    .await
                    .map_err(|e| PyralogError::StorageError(e.to_string()))?;

                Ok(remote_path.to_string_lossy().to_string())
            }
            RemoteStorageConfig::S3 { bucket, .. } => {
                // In production, use AWS SDK to upload to S3
                // For now, return a mock remote URL
                let filename = segment_path
                    .file_name()
                    .ok_or_else(|| PyralogError::StorageError("Invalid segment path".to_string()))?
                    .to_string_lossy();
                
                Ok(format!("s3://{}/{}", bucket, filename))
            }
            RemoteStorageConfig::Azure { container, .. } => {
                let filename = segment_path
                    .file_name()
                    .ok_or_else(|| PyralogError::StorageError("Invalid segment path".to_string()))?
                    .to_string_lossy();
                
                Ok(format!("azure://{}/{}", container, filename))
            }
            RemoteStorageConfig::Gcs { bucket, .. } => {
                let filename = segment_path
                    .file_name()
                    .ok_or_else(|| PyralogError::StorageError("Invalid segment path".to_string()))?
                    .to_string_lossy();
                
                Ok(format!("gs://{}/{}", bucket, filename))
            }
        }
    }

    /// Download a segment from remote storage
    pub async fn download_segment(&self, remote_url: &str, local_path: &Path) -> Result<()> {
        match &self.remote_config {
            RemoteStorageConfig::Local { .. } => {
                let remote_path = PathBuf::from(remote_url.trim_start_matches("file://"));
                
                fs::copy(&remote_path, local_path)
                    .await
                    .map_err(|e| PyralogError::StorageError(e.to_string()))?;

                Ok(())
            }
            _ => {
                // In production, implement download from cloud providers
                Err(PyralogError::StorageError(
                    "Remote download not yet implemented".to_string(),
                ))
            }
        }
    }

    /// Archive old segments based on retention policy
    pub async fn archive_old_segments(&self, before_offset: LogOffset) -> Result<Vec<String>> {
        let mut archived = Vec::new();

        let mut entries = fs::read_dir(&self.local_path)
            .await
            .map_err(|e| PyralogError::StorageError(e.to_string()))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| PyralogError::StorageError(e.to_string()))?
        {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) != Some("log") {
                continue;
            }

            if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(offset) = filename.parse::<u64>() {
                    if offset < before_offset.as_u64() {
                        let remote_url = self.upload_segment(&path).await?;
                        fs::remove_file(&path)
                            .await
                            .map_err(|e| PyralogError::StorageError(e.to_string()))?;
                        
                        // Also remove index file
                        let index_path = path.with_extension("index");
                        if index_path.exists() {
                            fs::remove_file(&index_path)
                                .await
                                .map_err(|e| PyralogError::StorageError(e.to_string()))?;
                        }

                        archived.push(remote_url);
                    }
                }
            }
        }

        Ok(archived)
    }
}

