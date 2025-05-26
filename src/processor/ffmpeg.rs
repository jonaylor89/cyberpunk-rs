use std::collections::HashMap;
use tracing::debug;
use color_eyre::Result;
use tempfile::TempDir;
use tokio::process::Command;
use tracing::instrument;

use crate::{
    blob::{AudioBuffer, AudioFormat},
    cyberpunkpath::params::Params,
};

#[instrument(skip(input, params, temp_dir))]
pub async fn process_audio(
    input: &AudioBuffer,
    params: &Params,
    temp_dir: TempDir,
    additional_tags: &HashMap<String, String>,
) -> Result<AudioBuffer> {
    let output_format = params.format.unwrap_or(AudioFormat::Mp3);

    let input_path = temp_dir
        .path()
        .join(format!("in.{}", input.format().extension()));
    let output_path = temp_dir
        .path()
        .join(format!("out.{}", output_format.extension()));

    // Write input file
    tokio::fs::write(&input_path, input.as_ref()).await?;

    // Build FFmpeg command
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-i", input_path.to_str().unwrap(), "-y"]);

    // Add optional metadata
    if let Some(tags) = &params.tags {
        for (k, v) in tags {
            cmd.args(["-metadata", &format!("{}={}", k, v)]);
        }
    }

    // Add additional tags
    for (k, v) in additional_tags {
        cmd.args(["-metadata", &format!("{}={}", k, v)]);
    }

    // Add encoding parameters and output path
    cmd.args(params.to_ffmpeg_args())
        .arg(output_path.to_str().unwrap());

    debug!(?cmd, "Executing FFmpeg command");

    // Execute FFmpeg
    let status = cmd.status().await?;
    if !status.success() {
        return Err(color_eyre::eyre::eyre!("FFmpeg failed"));
    }

    // Read and return output
    let processed = tokio::fs::read(&output_path).await?;
    Ok(AudioBuffer::from_bytes_with_format(
        processed,
        output_format,
    ))
}
