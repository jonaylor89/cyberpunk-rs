use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};

use crate::cyberpunkpath::params::Params;
use crate::state::AppStateDyn;

#[derive(Debug, Deserialize)]
pub struct McpRequest {
    method: String,
    params: Option<Params>,
}

#[derive(Debug, Serialize)]
pub struct McpResponse {
    result: Option<Value>,
    error: Option<McpError>,
}

#[derive(Debug, Serialize)]
pub struct McpError {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct ToolDescription {
    name: String,
    description: String,
    input_schema: Value,
    output_schema: Value,
}

pub async fn mcp_handler(
    State(_state): State<AppStateDyn>,
    Json(req): Json<McpRequest>,
) -> impl IntoResponse {
    info!("MCP request received: {:?}", req.method);

    match req.method.as_str() {
        "describe" => describe_capabilities(),
        "process_audio" => match req.params {
            Some(params) => (
                StatusCode::OK,
                Json(McpResponse {
                    result: Some(json!({
                        "url": format!("localhost:8000/{}", params),
                    })),
                    error: None,
                }),
            ),
            None => (
                StatusCode::BAD_REQUEST,
                Json(McpResponse {
                    result: None,
                    error: Some(McpError {
                        code: 400,
                        message: "Missing parameters".to_string(),
                    }),
                }),
            ),
        },
        _ => {
            error!("Unknown method: {}", req.method);
            (
                StatusCode::BAD_REQUEST,
                Json(McpResponse {
                    result: None,
                    error: Some(McpError {
                        code: 404,
                        message: format!("Unknown method: {}", req.method),
                    }),
                }),
            )
        }
    }
}

fn describe_capabilities() -> (StatusCode, Json<McpResponse>) {
    let input_schema = json!({
        "type": "object",
        "properties": {
            "audio_url": {
                "type": "string",
                "description": "URL or path to the audio file to process"
            },
            "format": {
                "type": "string",
                "enum": ["mp3", "wav", "ogg", "flac"],
                "description": "Output format of the processed audio"
            },
            "volume": {
                "type": "number",
                "description": "Volume adjustment multiplier"
            },
            "speed": {
                "type": "number",
                "description": "Playback speed multiplier"
            },
            "reverse": {
                "type": "boolean",
                "description": "Whether to reverse the audio"
            },
            "start_time": {
                "type": "number",
                "description": "Start time in seconds"
            },
            "duration": {
                "type": "number",
                "description": "Duration in seconds"
            },
            // Add more properties as needed
        },
        "required": ["audio_url"]
    });

    let output_schema = json!({
        "type": "object",
        "properties": {
            "processed_audio_url": {
                "type": "string",
                "description": "URL to the processed audio file"
            },
            "duration": {
                "type": "number",
                "description": "Duration of the processed audio in seconds"
            },
            "format": {
                "type": "string",
                "description": "Format of the processed audio"
            }
        }
    });

    let tools = vec![ToolDescription {
        name: "process_audio".to_string(),
        description: "Process audio files with various operations and effects".to_string(),
        input_schema,
        output_schema,
    }];

    (
        StatusCode::OK,
        Json(McpResponse {
            result: Some(json!({ "tools": tools })),
            error: None,
        }),
    )
}
