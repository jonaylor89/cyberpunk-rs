use secrecy::SecretString;
use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use tracing::error;

use crate::cyberpunkpath::normalize::SafeCharsType;

#[derive(serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub processor: ProcessorSettings,
    pub storage: StorageSettings,
    pub cache: CacheSettings,
}

#[derive(serde::Deserialize, Clone)]
#[serde(default)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub hmac_secret: SecretString,
}

impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            port: 8080,                                                      // default port
            host: String::from("127.0.0.1"),                                 // default host
            hmac_secret: SecretString::from("this-is-a-secret".to_string()), // empty secret
        }
    }
}

#[derive(serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct ProcessorSettings {
    pub disable_blur: bool,
    pub disabled_filters: Vec<String>,
    pub max_filter_ops: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<i32>,
    pub max_cache_files: i32,
    pub max_cache_mem: i32,
    pub max_cache_size: i32,
    pub max_width: i32,
    pub max_height: i32,
    pub max_resolution: i32,
    pub max_animation_frames: usize,
    pub strip_metadata: bool,
    pub avif_speed: i32,
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct StorageSettings {
    pub base_dir: String,
    pub path_prefix: String,
    pub safe_chars: SafeCharsType,
    pub client: StorageClient,
}

#[derive(Deserialize, Clone)]
pub enum StorageClient {
    S3(S3Settings),
    GCS(GCSSettings),
    Filesystem(FilesystemSettings),
}

impl Default for StorageClient {
    fn default() -> Self {
        Self::Filesystem(FilesystemSettings::default())
    }
}

#[derive(Deserialize, Clone)]
pub struct S3Settings {
    pub region: String,
    pub bucket: String,

    #[serde(default = "default_s3_endpoint")]
    pub endpoint: String,
    pub access_key: SecretString,
    pub secret_key: SecretString,
}

fn default_s3_endpoint() -> String {
    "https://s3.amazonaws.com".to_string()
}

#[derive(Deserialize, Clone)]
pub struct GCSSettings {
    pub bucket: String,
    pub credentials: SecretString,
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct FilesystemSettings {
    #[serde(default = "default_base_dir")]
    pub base_dir: String,
}

fn default_base_dir() -> String {
    "uploads".to_string()
}

#[derive(Deserialize, Clone)]
pub enum CacheSettings {
    Redis { uri: String },
    Filesystem(FilesystemCache),
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct FilesystemCache {
    #[serde(default = "default_cache_base_dir")]
    pub base_dir: String,
}

fn default_cache_base_dir() -> String {
    "cache".to_string()
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self::Filesystem(FilesystemCache::default())
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`",
                other
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("config");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let builder = config::Config::builder()
        .add_source(config::File::from(configuration_directory.join("base")).required(true))
        .add_source(
            config::File::from(configuration_directory.join(environment.as_str())).required(true),
        )
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        );

    builder
        .build()?
        .try_deserialize::<Settings>()
        .inspect_err(|e| error!("Failed to load configuration: {}", e))
}
