use axum::async_trait;
use color_eyre::Result;
use infer;

#[async_trait]
pub trait AudioStorage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Blob>;
    async fn put(&self, key: &str, blob: &Blob) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

// #[derive(Debug)]
// pub struct Stat {
//     pub size: u64,
//     pub modified: Option<time::SystemTime>,
// }

#[derive(Debug)]
pub struct Blob {
    pub data: Vec<u8>,
    pub content_type: String,
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl Blob {
    pub fn new(data: Vec<u8>) -> Self {
        let content_type = match infer::get(&data) {
            Some(kind) => kind.mime_type().to_string(),
            None => "application/octet-stream".to_string(),
        };

        Blob { data, content_type }
    }

    pub fn supports_animation(&self) -> bool {
        self.content_type.starts_with("image/gif") || self.content_type.starts_with("image/webp")
    }
}
