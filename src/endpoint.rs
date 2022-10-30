
use crate::transformations::concat::concat;

#[derive(Debug)]
pub enum Transformation {
    Concat(String),
}

#[derive(Debug)]
pub struct Endpoint<'endpoint> {
    pub audio: &'endpoint str,
    pub pipeline: Vec<Transformation>,
}

impl<'endpoint> Endpoint<'_> {
    pub fn new() -> Self {
        Endpoint {
            audio: "",
            pipeline: vec!(),
        }
    }

    pub fn process(&self) -> Result<(), String> {
        for transformation in &self.pipeline {
            let output = match transformation {
                Transformation::Concat(filename) => concat(self.audio, filename),
            };
            tracing::info!("{}", output);
        }

        Ok(())
    }
}