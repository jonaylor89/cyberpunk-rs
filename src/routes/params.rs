use axum::{http::StatusCode, Json};
use tracing::info;

use crate::cyberpunkpath::params::Params;

#[tracing::instrument]
pub async fn params(params: Params) -> Result<Json<Params>, (StatusCode, String)> {
    info!("params: {:?}", params);

    Ok(Json(params))
}
