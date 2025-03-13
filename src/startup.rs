use crate::cache::cache::AudioCache;
use crate::cache::redis::RedisCache;
use crate::config::{Settings, StorageClient};
use crate::cyberpunkpath::hasher::{suffix_result_storage_hasher, verify_hash};
use crate::cyberpunkpath::params::Params;
use crate::metrics::{setup_metrics_recorder, track_metrics};
use crate::middleware::cache_middleware;
use crate::processor::processor::{AudioProcessor, Processor};
use crate::state::AppStateDyn;
use crate::storage::file::FileStorage;
use crate::storage::gcs::GCloudStorage;
use crate::storage::s3::S3Storage;
use crate::storage::storage::{AudioStorage, Blob};
use axum::body::Body;
use axum::extract::{MatchedPath, Request, State};
use axum::http::{header, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{middleware, Json};
use axum::{serve::Serve, Router};
use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use reqwest;
use secrecy::ExposeSecret;
use std::future::ready;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task;
use tower_http::trace::TraceLayer;
use tracing::{info, info_span, warn};

pub struct Application {
    pub port: u16,
    server: Serve<Router, Router>,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self> {
        let address = format!("{}:{}", config.application.host, config.application.port);
        println!("Server started at {}\n", &address);
        let listener = TcpListener::bind(address).await.wrap_err(
            "Failed to bind to the port. Make sure you have the correct permissions to bind to the port",
        )?;
        let port = listener.local_addr()?.port();

        let processor = Processor::new(config.processor);
        let cache = RedisCache::new("redis://redis:6379")?;
        let server = match config.storage.client {
            StorageClient::S3(s3_settings) => {
                info!("Using S3 storage");
                let storage = S3Storage::new(
                    config.storage.base_dir,
                    config.storage.path_prefix,
                    config.storage.safe_chars,
                    s3_settings.endpoint,
                    s3_settings.region,
                    s3_settings.bucket,
                    s3_settings.access_key.expose_secret(),
                    s3_settings.secret_key.expose_secret(),
                )
                .await?;

                // Ensure bucket exists
                storage.ensure_bucket_exists().await?;

                run(listener, storage, processor, cache).await?
            }
            StorageClient::GCS(gcs_settings) => {
                info!("using GCS storage");
                let storage = GCloudStorage::new(
                    config.storage.base_dir,
                    config.storage.path_prefix,
                    config.storage.safe_chars,
                    gcs_settings.bucket,
                )
                .await;

                run(listener, storage, processor, cache).await?
            }
            StorageClient::Filesystem(filesystem_settings) => {
                info!("using filesystem storage");
                let storage = FileStorage::new(
                    PathBuf::from(filesystem_settings.base_dir),
                    config.storage.path_prefix,
                    config.storage.safe_chars,
                );

                run(listener, storage, processor, cache).await?
            }
        };

        Ok(Self { port, server })
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        println!(
            r#"\n
            _                                 _
  ___ _   _| |__   ___ _ __ _ __  _   _ _ __ | | __
 / __| | | | '_ \ / _ \ '__| '_ \| | | | '_ \| |/ /
| (__| |_| | |_) |  __/ |  | |_) | |_| | | | |   <
 \___|\__, |_.__/ \___|_|  | .__/ \__,_|_| |_|_|\_\
      |___/                |_|
        "#
        );
        self.server.await
    }
}

async fn run<S, P, C>(
    listener: TcpListener,
    storage: S,
    processor: P,
    cache: C,
) -> Result<Serve<Router, Router>>
where
    S: AudioStorage + Clone + Send + Sync + 'static,
    P: AudioProcessor + Send + Sync + 'static,
    C: AudioCache + Clone + Send + Sync + 'static,
{
    let recorder_handle = setup_metrics_recorder();

    let state = AppStateDyn {
        storage: Arc::new(storage.clone()),
        processor: Arc::new(processor),
        cache: Arc::new(cache.clone()),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(move || ready(recorder_handle.render())))
        .route("/", get(root))
        .route("/params/*cyberpunkpath", get(params))
        .route_layer(middleware::from_fn(track_metrics))
        .nest(
            "/",
            Router::new()
                .route("/*cyberpunkpath", get(handler))
                // add auth middleware for params and hash
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    cache_middleware,
                )),
        )
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                // Log the matched route's path (with placeholders not filled in).
                // Use request.uri() or OriginalUri if you want the real path.
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty,
                )
            }),
        )
        .with_state(state);

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    let server = axum::serve(listener, app);

    Ok(server)
}

#[tracing::instrument(skip(state))]
async fn handler(
    State(state): State<AppStateDyn>,
    params: Params,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("params: {:?}", params);

    // if let (Some(hash), Some(path)) = (&params.hash, &params.path) {
    //     verify_hash(hash.to_owned().into(), path.to_owned().into()).map_err(|e| {
    //         (
    //             StatusCode::BAD_REQUEST,
    //             format!("Failed to verify hash: {}", e),
    //         )
    //     })?;
    // }

    // TODO: check result bucket for audio and serve if found
    let params_hash = suffix_result_storage_hasher(&params);
    let result = state.storage.get(&params_hash).await.inspect_err(|_| {
        tracing::info!("no audio in results storage: {}", &params);
    });
    if let Ok(blob) = result {
        return Response::builder()
            .header(header::CONTENT_TYPE, blob.content_type)
            .body(Body::from(blob.data))
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to build response: {}", e),
                )
            });
    }

    // TODO: add config in the config to allow/disallow fetching audios from the internet
    let blob = if audio.starts_with("https://") || audio.starts_with("http://") {
        let raw_bytes = reqwest::get(audio)
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

        let content_type = infer::get(&raw_bytes)
            .map(|mime| mime.to_string())
            .unwrap_or("audio/mpeg3".to_string());

        Blob {
            data: raw_bytes,
            content_type,
        }
    } else {
        state.storage.get(audio).await.map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                format!("Failed to fetch audio: {}", e),
            )
        })?
    };

    let blob = task::spawn_blocking(move || {
        // Perform CPU-intensive operation
        state.processor.process(&blob, &params)
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("joining spawned task failed: {}", e),
        )
    })?
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to process audio: {}", e),
        )
    })?;

    // TODO: save audio to result bucket
    state.storage.put(&params_hash, &blob).await.map_err(|e| {
        warn!("Failed to save result audio [{}]: {}", &params_hash, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save result audio: {}", e),
        )
    })?;

    Response::builder()
        .header(header::CONTENT_TYPE, blob.content_type)
        .body(Body::from(blob.data))
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build response: {}", e),
            )
        })
}

#[tracing::instrument]
async fn params(params: Params) -> Result<Json<Params>, (StatusCode, String)> {
    info!("params: {:?}", params);

    Ok(Json(params))
}

#[tracing::instrument]
async fn root() -> &'static str {
    "Hello, World"
}

#[tracing::instrument]
async fn health_check() -> &'static str {
    tracing::info!("Health check called");
    "OK"
}
