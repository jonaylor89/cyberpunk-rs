use crate::state::AppStateDyn;
use axum::http::{header, Response, StatusCode};
use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use std::time::Duration;

#[tracing::instrument(skip(state, req, next))]
pub async fn cache_middleware(
    State(state): State<AppStateDyn>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cache_key = format!("{}:{}", req.method(), req.uri().path());

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
        .set(&cache_key, bytes.as_ref(), Some(Duration::from_secs(3_600))) // 1 hour
        .await;

    Ok(Response::from_parts(parts, Body::from(bytes)))
}

// pub async fn auth_middleware(
//     State(state): State<AppStateDyn>,
//     req: Request,
//     next: Next,
// ) -> Result<impl IntoResponse, (StatusCode, String)> {
//     todo!()
// }
