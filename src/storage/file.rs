use crate::blob::AudioBuffer;
use crate::cyberpunkpath::normalize::{normalize, SafeCharsType};
use crate::storage::storage::AudioStorage;
use axum::async_trait;
use color_eyre::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct FileStorage {
    pub base_dir: PathBuf,
    pub path_prefix: String,
    pub safe_chars: SafeCharsType,
}

#[async_trait]
impl AudioStorage for FileStorage {
    #[tracing::instrument(skip(self))]
    async fn get(&self, key: &str) -> Result<AudioBuffer> {
        let full_path = self.get_full_path(key);
        let mut file = File::open(full_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        Ok(AudioBuffer::from_bytes(buffer))
    }

    #[tracing::instrument(skip(self, blob))]
    async fn put(&self, key: &str, blob: &AudioBuffer) -> Result<()> {
        let full_path = self.get_full_path(key);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(full_path)
            .await?;
        file.write_all(blob.as_ref()).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, key: &str) -> Result<()> {
        let full_path = self.get_full_path(key);
        tokio::fs::remove_file(full_path).await?;
        Ok(())
    }
}

impl FileStorage {
    pub fn new(base_dir: PathBuf, path_prefix: String, safe_chars: SafeCharsType) -> Self {
        FileStorage {
            base_dir,
            path_prefix,
            safe_chars,
        }
    }

    pub fn get_full_path(&self, key: &str) -> PathBuf {
        let safe_key = normalize(key, &self.safe_chars);
        self.base_dir
            .join(Path::new(&self.path_prefix))
            .join(safe_key)
    }
}
