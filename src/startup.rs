use crate::cache::cache::AudioCache;
use crate::cache::cache::Cache;
use crate::config::{Settings, StorageClient};
use crate::metrics::{setup_metrics_recorder, track_metrics};
use crate::middleware::auth_middleware;
use crate::middleware::cache_middleware;
use crate::processor::processor::{AudioProcessor, Processor};
use crate::routes::cyberpunkpath::cyberpunkpath_handler;
use crate::routes::health::health_check;
use crate::routes::mcp::mcp_handler;
use crate::routes::params::params;
use crate::routes::root::root_handler;
use crate::state::AppStateDyn;
use crate::storage::file::FileStorage;
use crate::storage::gcs::GCloudStorage;
use crate::storage::s3::S3Storage;
use crate::storage::storage::AudioStorage;
use crate::tags::create_tags;
use axum::extract::{MatchedPath, Request};
use axum::middleware;
use axum::routing::get;
use axum::routing::post;
use axum::{serve::Serve, Router};
use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use secrecy::ExposeSecret;
use std::future::ready;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, info_span};

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

        let additional_tags = create_tags(config.custom_tags)?;

        let processor = Processor::new(config.processor, additional_tags);
        let cache = Cache::new(config.cache)?;

        let server = match config.storage.client {
            Some(StorageClient::S3(s3_settings)) => {
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
            Some(StorageClient::GCS(gcs_settings)) => {
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
            None => {
                info!("using filesystem storage");
                let storage = FileStorage::new(
                    PathBuf::from(config.storage.base_dir),
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
        .route("/", get(root_handler))
        .route("/mcp", post(mcp_handler))
        .route("/params/*cyberpunkpath", get(params))
        .route_layer(middleware::from_fn(track_metrics))
        .nest(
            "/",
            Router::new()
                .route("/*cyberpunkpath", get(cyberpunkpath_handler))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
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

    debug!("listening on {}", listener.local_addr().unwrap());
    let server = axum::serve(listener, app);

    Ok(server)
}
