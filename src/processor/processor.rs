use std::num::NonZeroUsize;

use axum::async_trait;
use color_eyre::Result;
use tempfile::TempDir;
use tokio::sync::Semaphore;
use tracing::{info, instrument};

use crate::{
    blob::AudioBuffer, config::ProcessorSettings, cyberpunkpath::params::Params,
    processor::ffmpeg::process_audio,
};

#[async_trait]
pub trait AudioProcessor: Send + Sync {
    async fn process(&self, blob: &AudioBuffer, params: &Params) -> Result<AudioBuffer>;
}

#[derive(Debug)]
pub struct Processor {
    semaphore: Semaphore,
}

#[async_trait]
impl AudioProcessor for Processor {
    #[tracing::instrument(skip(self, blob, params))]
    async fn process(&self, blob: &AudioBuffer, params: &Params) -> Result<AudioBuffer> {
        let _permit = self.semaphore.acquire().await?;
        info!(params = ?params, "Processing with FFmpeg");

        let temp_dir = TempDir::new()?;

        let processed_audio = process_audio(blob, params, temp_dir).await?;
        info!("Audio processing completed successfully");

        Ok(processed_audio)
    }
}

impl Processor {
    #[instrument(skip(config))]
    pub fn new(config: ProcessorSettings) -> Self {
        let max_concurrent = config
            .concurrency
            .map(|concurrency| {
                NonZeroUsize::new(concurrency).expect("Concurrency should be non-zero")
            })
            .unwrap_or_else(|| {
                NonZeroUsize::new(num_cpus::get())
                    .expect("Number of CPUs should always be non-zero")
            });

        info!(
            max_concurrent = max_concurrent.get(),
            "Initializing processor"
        );

        Self {
            semaphore: Semaphore::new(max_concurrent.get()),
        }
    }
}
