use axum::{
    body::Body,
    extract::State,
    http::{header, Response, StatusCode},
    response::IntoResponse,
};
use tracing::{info, instrument, warn};

use crate::{
    blob::AudioBuffer,
    cyberpunkpath::{hasher::suffix_result_storage_hasher, params::Params},
    state::AppStateDyn,
};

#[instrument(skip(state))]
pub async fn cyberpunkpath_handler(
    State(state): State<AppStateDyn>,
    params: Params,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let params_hash = suffix_result_storage_hasher(&params);
    let result = state.storage.get(&params_hash).await.inspect_err(|_| {
        info!("no audio in results storage: {}", &params);
    });
    if let Ok(blob) = result {
        return Response::builder()
            .header(header::CONTENT_TYPE, blob.mime_type())
            .body(Body::from(blob.into_bytes()))
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to build response: {}", e),
                )
            });
    }

    let blob = if params.key.starts_with("https://") || params.key.starts_with("http://") {
        let raw_bytes = reqwest::get(&params.key)
            .await
            .map_err(|e| {
                (
                    StatusCode::NOT_FOUND,
                    format!("Failed to fetch audio: {}", e),
                )
            })?
            .bytes()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to fetch audio: {}", e),
                )
            })?
            .to_vec();

        AudioBuffer::from_bytes(raw_bytes)
    } else {
        state.storage.get(&params.key).await.map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                format!("Failed to fetch audio: {}", e),
            )
        })?
    };

    let processed_blob = state.processor.process(&blob, &params).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to process audio: {}", e),
        )
    })?;

    state
        .storage
        .put(&params_hash, &processed_blob)
        .await
        .map_err(|e| {
            warn!("Failed to save result audio [{}]: {}", &params_hash, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to save result audio: {}", e),
            )
        })?;

    Response::builder()
        .header(header::CONTENT_TYPE, processed_blob.mime_type())
        .body(Body::from(processed_blob.into_bytes()))
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build response: {}", e),
            )
        })
}
