use color_eyre::Result;

use crate::{config::ProcessorSettings, cyberpunkpath::params::Params, storage::storage::Blob};

pub trait AudioProcessor: Send + Sync {
    fn startup(&self) -> Result<()>;
    fn process(&self, blob: &Blob, params: &Params) -> Result<Blob>;
    fn shutdown(&self) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct Processor {}

impl AudioProcessor for Processor {
    fn startup(&self) -> Result<()> {
        Ok(())
    }

    fn process(&self, blob: &Blob, params: &Params) -> Result<Blob> {
        Ok(blob.to_owned())
    }

    fn shutdown(&self) -> Result<()> {
        Ok(())

}

impl Processor {
    pub fn new(_settings: ProcessorSettings) -> Self {
        Self {}
    }
}
