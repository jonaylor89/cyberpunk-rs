use color_eyre::Result;

use crate::{cyberpunkpath::params::Params, storage::storage::Blob};

pub trait AudioProcessor: Send + Sync {
    fn startup(&self) -> Result<()>;
    fn process(&self, blob: &Blob, params: &Params) -> Result<Blob>;
    fn shutdown(&self) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct Processor {}
