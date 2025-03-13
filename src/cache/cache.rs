use axum::async_trait;
use color_eyre::Result;
use std::time::Duration;

#[async_trait]
pub trait AudioCache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}
