use crate::cyberpunkpath::hasher::verify_hash;
use crate::cyberpunkpath::params::Params;
use crate::state::AppStateDyn;
use axum::http::{header, Response, StatusCode};
use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use std::time::Duration;

const CACHE_KEY_PREFIX: &str = "req_cache:";
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

#[tracing::instrument(skip(state, req, next))]
pub async fn cache_middleware(
    State(state): State<AppStateDyn>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cache_key = format!("{}:{}:{}", CACHE_KEY_PREFIX, req.method(), req.uri().path());

    let cache_response = state.cache.get(&cache_key).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get cache: {}", e),
        )
    })?;
    if let Some(buf) = cache_response {
        // Return cached response if available
        let content_type = infer::get(&buf)
            .map(|mime| mime.to_string())
            .unwrap_or("audio/mpeg3".to_string());
        let res = Response::builder()
            .header(header::CONTENT_TYPE, content_type)
            .body(Body::from(buf))
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to build response: {}", e),
                )
            })?;

        return Ok(res);
    }

    // If not cached, proceed with the request
    let response = next.run(req).await;
    if response.status() != StatusCode::OK {
        return Ok(response);
    }

    // Cache the response
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read response body: {}", e),
        )
    })?;

    // TODO: use hash key for this
    let _ = state
        .cache
        .set(&cache_key, bytes.as_ref(), Some(CACHE_TTL))
        .await;

    Ok(Response::from_parts(parts, Body::from(bytes)))
}

pub async fn auth_middleware(
    State(_): State<AppStateDyn>,
    params: Params,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let path = params.to_string();
    let hash = ""; // TODO: get hash from request path

    verify_hash(hash.to_owned().into(), path.to_owned().into()).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to verify hash: {}", e),
        )
    })?;

    Ok(next.run(req).await)
}
