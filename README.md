# Cyberpunk-rs

Advanced Audio Processing Server written in Rust

![GitHub](https://img.shields.io/github/license/jonaylor89/cyberpunk?logo=MIT) ![GitHub Workflow Status](https://img.shields.io/github/workflow/status/jonaylor89/cyberpunk/Docker)

[![Run on Google Cloud](https://deploy.cloud.run/button.svg)](https://deploy.cloud.run?git_repo=https://github.com/jonaylor89/cyberpunk)

## Quick Start

```sh
docker run -p 8080:8080 -e PORT=8080 ghcr.io/jonaylor89/cyberpunk:main
```

Original audio test file:
```
https://raw.githubusercontent.com/jonaylor89/cyberpunk/main/testdata/celtic_pt2.mp3
```

Try out the following audio manipulation requests:
```
http://localhost:8080/unsafe/https://raw.githubusercontent.com/jonaylor89/cyberpunk/main/testdata/celtic_pt2.mp3
http://localhost:8080/unsafe/https://raw.githubusercontent.com/jonaylor89/cyberpunk/main/testdata/celtic_pt2.mp3?reverse=true
http://localhost:8080/unsafe/https://raw.githubusercontent.com/jonaylor89/cyberpunk/main/testdata/celtic_pt2.mp3?start_time=0&duration=10
http://localhost:8080/unsafe/https://raw.githubusercontent.com/jonaylor89/cyberpunk/main/testdata/celtic_pt2.mp3?reverse=true&fade_in=1&fade_out=1&speed=0.8
```

## Cyberpunk API

The Cyberpunk endpoint follows this URL structure:

```
/HASH|unsafe/AUDIO?param1=value1&param2=value2&...
```

- `HASH` is the URL signature hash, or `unsafe` if unsafe mode is used
- `AUDIO` is the audio URI (local file or remote URL)

### Supported Parameters

Cyberpunk supports a wide range of audio processing capabilities:

#### Format & Encoding
- `format` - Output format (mp3, wav, etc.)
- `codec` - Audio codec
- `sample_rate` - Sample rate in Hz
- `channels` - Number of audio channels
- `bit_rate` - Bit rate in kbps
- `bit_depth` - Bit depth
- `quality` - Encoding quality (0.0-1.0)
- `compression_level` - Compression level

#### Time Operations
- `start_time` - Start time in seconds
- `duration` - Duration in seconds
- `speed` - Playback speed multiplier
- `reverse` - Reverse audio (true/false)

#### Volume Operations
- `volume` - Volume adjustment multiplier
- `normalize` - Normalize audio levels (true/false)
- `normalize_level` - Target normalization level in dB

#### Audio Effects
- `lowpass` - Lowpass filter cutoff frequency
- `highpass` - Highpass filter cutoff frequency
- `bandpass` - Bandpass filter parameters
- `bass` - Bass boost/cut level
- `treble` - Treble boost/cut level
- `echo` - Echo effect parameters
- `reverb` - Reverb effect parameters
- `chorus` - Chorus effect parameters
- `flanger` - Flanger effect parameters
- `phaser` - Phaser effect parameters
- `tremolo` - Tremolo effect parameters
- `compressor` - Compressor effect parameters
- `noise_reduction` - Noise reduction parameters

#### Fades
- `fade_in` - Fade in duration in seconds
- `fade_out` - Fade out duration in seconds
- `cross_fade` - Cross-fade duration in seconds

#### Advanced
- `custom_filters` - Custom FFmpeg filter parameters
- `custom_options` - Custom FFmpeg options
- `tags` - Metadata tags (as `tag_NAME=VALUE`)

### Preview Parameters with `/params`

You can preview the parameters for any request by adding `/params` before the endpoint:

```sh
curl "http://localhost:8080/params/unsafe/celtic_pt2.mp3?reverse=true&fade_in=1"

{
  "audio": "celtic_pt2.mp3",
  "reverse": true,
  "fade_in": 1.0
}
```

## Storage Options

Cyberpunk supports multiple storage backends:

- ✅ Local File System
- ✅ AWS S3 / MinIO
- ✅ Google Cloud Storage (GCS)

## Advanced Features

- ✅ Audio format conversion
- ✅ Audio time manipulation (slicing, speed, reverse)
- ✅ Audio effects and filters
- ✅ Audio fades
- ✅ Request caching
- ✅ Storage abstraction
- ✅ Metrics and monitoring
- ✅ Remote audio fetching

## Environment Configuration

Refer to the `.env` file for all configurable environment variables.

## Docker Compose Examples

### Local Storage Setup

```yaml
version: "3"
services:
  cyberpunk:
    image: jonaylor/cyberpunk:main
    volumes:
      - ./:/mnt/data
    environment:
      PORT: 8080
      AUDIO_PATH: "local"
      FILE_STORAGE_BASE_DIR: /mnt/data/testdata/
    ports:
      - "8080:8080"
```

### AWS S3 Storage Setup

```yaml
version: "3"
services:
  cyberpunk:
    image: jonaylor/cyberpunk:main
    environment:
      PORT: 8080
      CYBERPUNK_SECRET: mysecret
      AWS_ACCESS_KEY_ID: ...
      AWS_SECRET_ACCESS_KEY: ...
      AWS_REGION: ...
      AUDIO_PATH: "s3"
      S3_LOADER_BUCKET: mybucket
      S3_LOADER_BASE_DIR: audio
      S3_STORAGE_BUCKET: mybucket
      S3_STORAGE_BASE_DIR: audio
      S3_RESULT_STORAGE_BUCKET: mybucket
      S3_RESULT_STORAGE_BASE_DIR: audio/result
    ports:
      - "8080:8080"
```

### Google Cloud Storage Setup

```yaml
version: "3"
services:
  cyberpunk:
    image: jonaylor/cyberpunk:main
    environment:
      PORT: 8080
      CYBERPUNK_SECRET: mysecret
      AUDIO_PATH: "gcs"
      GCS_BUCKET: mybucket
      # Ensure appropriate GCP credentials are available
    ports:
      - "8080:8080"
```
