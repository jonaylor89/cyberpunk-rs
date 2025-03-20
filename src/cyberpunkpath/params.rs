use std::{
    collections::HashMap,
    fmt::{self, Display},
    str::FromStr,
};

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::blob::AudioFormat;

#[derive(Debug)]
pub struct CyberpunkPath {
    pub path: String,
}

pub trait Signer {
    fn sign(&self, path: &str) -> String;
}

#[async_trait]
impl<S> FromRequestParts<S> for Params
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    #[tracing::instrument(skip(parts, _state))]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Access the URI and perform your custom parsing logic
        let uri = &parts.uri;
        let path = uri.path().trim_start_matches("/params");

        info!("Parsing path: {}", path);

        let params = Params::from_str(path).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to parse params: {}", e),
            )
        })?;

        Ok(params)
    }
}

impl TryFrom<&str> for Params {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value).map_err(|e| format!("Failed to parse path: {}", e))
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Params {
    // the uri for the audio
    #[serde(skip)]
    pub audio: String,

    // Audio Format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<AudioFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_depth: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_level: Option<i32>,

    // Time Operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,

    // Volume Operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalize: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalize_level: Option<f64>,

    // Filters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lowpass: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highpass: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandpass: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bass: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub treble: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverb: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chorus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flanger: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phaser: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tremolo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise_reduction: Option<String>,

    // Fade Operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fade_in: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fade_out: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_fade: Option<f64>,

    // Advanced
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_filters: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_options: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, String>>,
}

impl Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let query_params = self.to_query();
        let query_str = query_params
            .iter()
            .map(|(k, v)| {
                v.iter()
                    .map(|val| format!("{}={}", k, urlencoding::encode(val)))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>()
            .join("&");

        write!(f, "{}?{}", self.audio, query_str)
    }
}

impl FromStr for Params {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('?').collect();
        let path = parts[0].trim_start_matches('/');

        // Split path into components and take the last one as audio id
        let path_components: Vec<&str> = path.split('/').collect();
        let audio = path_components.last().unwrap_or(&"").to_string();

        let mut query = HashMap::new();
        if parts.len() > 1 {
            // Parse query parameters
            for param in parts[1].split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    query
                        .entry(key.to_string())
                        .or_insert_with(Vec::new)
                        .push(urlencoding::decode(value)?.into_owned());
                }
            }
        }

        Self::from_path(audio, query)
    }
}

impl Params {
    pub fn from_path(audio: String, query: HashMap<String, Vec<String>>) -> Result<Self> {
        let mut params = Self::default();
        params.audio = audio;

        for (key, values) in query {
            if let Some(value) = values.first() {
                match key.as_str() {
                    "format" => {
                        params.format =
                            Some(value.parse::<AudioFormat>().unwrap_or(AudioFormat::Mp3))
                    }
                    "codec" => params.codec = Some(value.to_string()),
                    "sample_rate" => params.sample_rate = value.parse().ok(),
                    "channels" => params.channels = value.parse().ok(),
                    "bit_rate" => params.bit_rate = value.parse().ok(),
                    "bit_depth" => params.bit_depth = value.parse().ok(),
                    "quality" => params.quality = value.parse().ok(),
                    "compression_level" => params.compression_level = value.parse().ok(),
                    "start_time" => params.start_time = value.parse().ok(),
                    "duration" => params.duration = value.parse().ok(),
                    "speed" => params.speed = value.parse().ok(),
                    "reverse" => params.reverse = Some(value == "true" || value == "1"),
                    "volume" => params.volume = value.parse().ok(),
                    "normalize" => params.normalize = Some(value == "true" || value == "1"),
                    "normalize_level" => params.normalize_level = value.parse().ok(),
                    "lowpass" => params.lowpass = value.parse().ok(),
                    "highpass" => params.highpass = value.parse().ok(),
                    "bandpass" => params.bandpass = Some(value.to_string()),
                    "bass" => params.bass = value.parse().ok(),
                    "treble" => params.treble = value.parse().ok(),
                    "echo" => params.echo = Some(value.to_string()),
                    "reverb" => params.reverb = Some(value.to_string()),
                    "chorus" => params.chorus = Some(value.to_string()),
                    "flanger" => params.flanger = Some(value.to_string()),
                    "phaser" => params.phaser = Some(value.to_string()),
                    "tremolo" => params.tremolo = Some(value.to_string()),
                    "compressor" => params.compressor = Some(value.to_string()),
                    "noise_reduction" => params.noise_reduction = Some(value.to_string()),
                    "fade_in" => params.fade_in = value.parse().ok(),
                    "fade_out" => params.fade_out = value.parse().ok(),
                    "cross_fade" => params.cross_fade = value.parse().ok(),
                    "custom_filters" => params.custom_filters = Some(values.clone()),
                    "custom_options" => params.custom_options = Some(values.clone()),
                    _ => {
                        if key.starts_with("tag_") {
                            let tag_key = key.trim_start_matches("tag_").to_string();
                            params
                                .tags
                                .get_or_insert_with(HashMap::new)
                                .insert(tag_key, value.to_string());
                        }
                    }
                }
            }
        }

        Ok(params)
    }

    pub fn to_query(&self) -> HashMap<String, Vec<String>> {
        let mut query: HashMap<String, Vec<String>> = HashMap::new();

        if let Some(format) = &self.format {
            query.insert("format".to_string(), vec![format.to_string()]);
        }
        if let Some(codec) = &self.codec {
            query.insert("codec".to_string(), vec![codec.clone()]);
        }
        if let Some(rate) = self.sample_rate {
            query.insert("sample_rate".to_string(), vec![rate.to_string()]);
        }
        if let Some(channels) = self.channels {
            query.insert("channels".to_string(), vec![channels.to_string()]);
        }
        if let Some(rate) = self.bit_rate {
            query.insert("bit_rate".to_string(), vec![rate.to_string()]);
        }
        if let Some(depth) = self.bit_depth {
            query.insert("bit_depth".to_string(), vec![depth.to_string()]);
        }
        if let Some(quality) = self.quality {
            query.insert("quality".to_string(), vec![quality.to_string()]);
        }
        if let Some(level) = self.compression_level {
            query.insert("compression_level".to_string(), vec![level.to_string()]);
        }
        if let Some(time) = self.start_time {
            query.insert("start_time".to_string(), vec![time.to_string()]);
        }
        if let Some(duration) = self.duration {
            query.insert("duration".to_string(), vec![duration.to_string()]);
        }
        if let Some(speed) = self.speed {
            query.insert("speed".to_string(), vec![speed.to_string()]);
        }
        if let Some(reverse) = self.reverse {
            query.insert("reverse".to_string(), vec![reverse.to_string()]);
        }
        if let Some(volume) = self.volume {
            query.insert("volume".to_string(), vec![volume.to_string()]);
        }
        if let Some(normalize) = self.normalize {
            query.insert("normalize".to_string(), vec![normalize.to_string()]);
        }
        if let Some(level) = self.normalize_level {
            query.insert("normalize_level".to_string(), vec![level.to_string()]);
        }
        if let Some(freq) = self.lowpass {
            query.insert("lowpass".to_string(), vec![freq.to_string()]);
        }
        if let Some(freq) = self.highpass {
            query.insert("highpass".to_string(), vec![freq.to_string()]);
        }
        if let Some(band) = &self.bandpass {
            query.insert("bandpass".to_string(), vec![band.clone()]);
        }
        if let Some(bass) = self.bass {
            query.insert("bass".to_string(), vec![bass.to_string()]);
        }
        if let Some(treble) = self.treble {
            query.insert("treble".to_string(), vec![treble.to_string()]);
        }
        if let Some(echo) = &self.echo {
            query.insert("echo".to_string(), vec![echo.clone()]);
        }
        if let Some(reverb) = &self.reverb {
            query.insert("reverb".to_string(), vec![reverb.clone()]);
        }
        if let Some(chorus) = &self.chorus {
            query.insert("chorus".to_string(), vec![chorus.clone()]);
        }
        if let Some(flanger) = &self.flanger {
            query.insert("flanger".to_string(), vec![flanger.clone()]);
        }
        if let Some(phaser) = &self.phaser {
            query.insert("phaser".to_string(), vec![phaser.clone()]);
        }
        if let Some(tremolo) = &self.tremolo {
            query.insert("tremolo".to_string(), vec![tremolo.clone()]);
        }
        if let Some(compressor) = &self.compressor {
            query.insert("compressor".to_string(), vec![compressor.clone()]);
        }
        if let Some(nr) = &self.noise_reduction {
            query.insert("noise_reduction".to_string(), vec![nr.clone()]);
        }
        if let Some(fade) = self.fade_in {
            query.insert("fade_in".to_string(), vec![fade.to_string()]);
        }
        if let Some(fade) = self.fade_out {
            query.insert("fade_out".to_string(), vec![fade.to_string()]);
        }
        if let Some(fade) = self.cross_fade {
            query.insert("cross_fade".to_string(), vec![fade.to_string()]);
        }
        if let Some(filters) = &self.custom_filters {
            query.insert("custom_filters".to_string(), filters.clone());
        }
        if let Some(options) = &self.custom_options {
            query.insert("custom_options".to_string(), options.clone());
        }
        if let Some(tags) = &self.tags {
            for (key, value) in tags {
                query.insert(format!("tag_{}", key), vec![value.clone()]);
            }
        }

        query
    }

    pub fn to_ffmpeg_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(format) = &self.format {
            args.extend_from_slice(&["-f".to_string(), format.to_string()]);
        }
        if let Some(codec) = &self.codec {
            args.extend_from_slice(&["-c:a".to_string(), codec.clone()]);
        }
        if let Some(rate) = self.sample_rate {
            args.extend_from_slice(&["-ar".to_string(), rate.to_string()]);
        }
        if let Some(channels) = self.channels {
            args.extend_from_slice(&["-ac".to_string(), channels.to_string()]);
        }
        if let Some(rate) = self.bit_rate {
            args.extend_from_slice(&["-b:a".to_string(), format!("{}k", rate)]);
        }
        if let Some(quality) = self.quality {
            args.extend_from_slice(&["-q:a".to_string(), format!("{:.1}", quality)]);
        }
        if let Some(level) = self.compression_level {
            args.extend_from_slice(&["-compression_level".to_string(), level.to_string()]);
        }
        if let Some(time) = self.start_time {
            args.extend_from_slice(&["-ss".to_string(), format!("{:.3}", time)]);
        }
        if let Some(duration) = self.duration {
            args.extend_from_slice(&["-t".to_string(), format!("{:.3}", duration)]);
        }

        let filters = self.collect_filters();
        if !filters.is_empty() {
            args.extend_from_slice(&["-filter:a".to_string(), filters.join(",")]);
        }

        if let Some(options) = &self.custom_options {
            args.extend(options.iter().cloned());
        }

        args
    }

    fn collect_filters(&self) -> Vec<String> {
        let mut filters = Vec::new();

        if let Some(speed) = self.speed {
            if speed != 1.0 {
                filters.push(format!("atempo={:.3}", speed));
            }
        }
        if let Some(true) = self.reverse {
            filters.push("areverse".to_string());
        }
        if let Some(volume) = self.volume {
            if volume != 1.0 {
                filters.push(format!("volume={:.2}", volume));
            }
        }
        if let Some(true) = self.normalize {
            let level = self.normalize_level.unwrap_or(0.0);
            filters.push(format!("loudnorm=I={:.1}", level));
        }
        if let Some(freq) = self.lowpass {
            filters.push(format!("lowpass=f={:.1}", freq));
        }
        if let Some(freq) = self.highpass {
            filters.push(format!("highpass=f={:.1}", freq));
        }
        if let Some(band) = &self.bandpass {
            filters.push(format!("bandpass={}", band));
        }
        if let Some(bass) = self.bass {
            filters.push(format!("bass=g={:.1}", bass));
        }
        if let Some(treble) = self.treble {
            filters.push(format!("treble=g={:.1}", treble));
        }
        if let Some(echo) = &self.echo {
            filters.push(format!("aecho={}", echo));
        }
        if let Some(reverb) = &self.reverb {
            filters.push(format!("averberate={}", reverb));
        }
        if let Some(chorus) = &self.chorus {
            filters.push(format!("chorus={}", chorus));
        }
        if let Some(flanger) = &self.flanger {
            filters.push(format!("flanger={}", flanger));
        }
        if let Some(phaser) = &self.phaser {
            filters.push(format!("aphaser={}", phaser));
        }
        if let Some(tremolo) = &self.tremolo {
            filters.push(format!("tremolo={}", tremolo));
        }
        if let Some(compressor) = &self.compressor {
            filters.push(format!("acompressor={}", compressor));
        }
        if let Some(nr) = &self.noise_reduction {
            filters.push(format!("anlmdn={}", nr));
        }
        if let Some(fade) = self.fade_in {
            filters.push(format!("afade=t=in:d={:.3}", fade));
        }
        if let Some(fade) = self.fade_out {
            filters.push(format!("afade=t=out:d={:.3}", fade));
        }
        if let Some(fade) = self.cross_fade {
            filters.push(format!("acrossfade=d={:.3}", fade));
        }

        if let Some(custom_filters) = &self.custom_filters {
            filters.extend(custom_filters.clone());
        }

        filters
    }

    pub fn to_unsafe_string(p: &Params) -> String {
        let img_path = p.to_string();
        format!("unsafe/{}", img_path)
    }

    pub fn to_signed_string<S: Signer>(p: &Params, signer: S) -> String {
        let img_path = p.to_string();
        format!("{}/{}", signer.sign(&img_path), img_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_display() {
        let params = Params {
            audio: "test.mp3".to_string(),
            format: Some(AudioFormat::Mp3),
            quality: Some(0.5),
            ..Default::default()
        };
        assert_eq!(params.to_string(), "Audio: test.mp3, Format: mp3");
    }

    // Add more tests as needed
}
