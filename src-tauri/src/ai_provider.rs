/// Ollama provider adapter for local AI inference.
///
/// Authority: AI is a candidate generator, not a final authority.
/// Deterministic rule validation must follow any AI output.

use serde::Deserialize;
use std::process::Command as StdCommand;

use crate::db::{ModelCapabilities, OllamaHealthResult, OllamaModel};

const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<TagsModel>,
}

#[derive(Debug, Deserialize)]
struct TagsModel {
    name: String,
    size: u64,
    details: Option<TagsModelDetails>,
}

#[derive(Debug, Deserialize)]
struct TagsModelDetails {
    parameter_size: Option<String>,
    family: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ShowResponse {
    #[serde(default)]
    capabilities: Vec<String>,
}

/// Check Ollama health by calling its root endpoint.
pub fn check_health(base_url: &str) -> OllamaHealthResult {
    let url = if base_url.is_empty() { DEFAULT_OLLAMA_URL } else { base_url };
    let output = StdCommand::new("curl")
        .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", "--connect-timeout", "3", url])
        .output();

    match output {
        Ok(o) => {
            let status = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if status == "200" {
                OllamaHealthResult {
                    healthy: true, version: get_version(url),
                    base_url: url.to_string(), error: None,
                }
            } else {
                OllamaHealthResult {
                    healthy: false, version: String::new(),
                    base_url: url.to_string(), error: Some(format!("HTTP status: {}", status)),
                }
            }
        }
        Err(e) => OllamaHealthResult {
            healthy: false, version: String::new(),
            base_url: url.to_string(), error: Some(format!("Connection failed: {}", e)),
        },
    }
}

fn get_version(base_url: &str) -> String {
    let url = format!("{}/api/version", base_url);
    StdCommand::new("curl").args(["-s", "--connect-timeout", "3", &url]).output()
        .ok()
        .and_then(|o| {
            let body = String::from_utf8_lossy(&o.stdout);
            body.find(r#""version":""#).and_then(|start| {
                let rest = &body[start + 11..];
                rest.find('"').map(|end| rest[..end].to_string())
            })
        })
        .unwrap_or_default()
}

/// List installed Ollama models with metadata.
pub fn list_models(base_url: &str) -> Result<Vec<OllamaModel>, String> {
    let url = if base_url.is_empty() { DEFAULT_OLLAMA_URL } else { base_url };
    let api_url = format!("{}/api/tags", url);
    let output = StdCommand::new("curl")
        .args(["-s", "--connect-timeout", "5", &api_url])
        .output()
        .map_err(|e| format!("Failed to call Ollama: {}", e))?;

    let body = String::from_utf8_lossy(&output.stdout);
    let response: TagsResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse model list: {}", e))?;

    Ok(response.models.into_iter().map(|m| {
        let details = m.details.unwrap_or(TagsModelDetails { parameter_size: None, family: None });
        OllamaModel {
            name: m.name.clone(),
            size_bytes: m.size,
            parameter_size: details.parameter_size.unwrap_or_default(),
            family: details.family.unwrap_or_default(),
            capabilities: detect_capabilities_from_name(&m.name),
        }
    }).collect())
}

/// Heuristic capability detection from model name.
fn detect_capabilities_from_name(name: &str) -> ModelCapabilities {
    let lower = name.to_lowercase();
    let vision = lower.contains("llava") || lower.contains("vision")
        || lower.contains("bakllava") || lower.contains("minicpm-v")
        || lower.contains("moondream") || lower.contains("internvl");
    ModelCapabilities { vision, text_generation: true, structured_output: true }
}

/// Get detailed capabilities via /api/show.
pub fn get_model_capabilities(base_url: &str, model_name: &str) -> Result<ModelCapabilities, String> {
    let url = if base_url.is_empty() { DEFAULT_OLLAMA_URL } else { base_url };
    let api_url = format!("{}/api/show", url);
    let body_json = format!(r#"{{"name":"{}"}}"#, model_name);
    let output = StdCommand::new("curl")
        .args(["-s", "--connect-timeout", "5", "-X", "POST",
               "-H", "Content-Type: application/json", "-d", &body_json, &api_url])
        .output()
        .map_err(|e| format!("Failed to call Ollama show: {}", e))?;

    let body = String::from_utf8_lossy(&output.stdout);
    let response: ShowResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse show response: {}", e))?;

    Ok(ModelCapabilities {
        vision: response.capabilities.contains(&"vision".to_string()),
        text_generation: true, structured_output: true,
    })
}

/// Check if a model supports vision.
/// Used by Phase 5+ image analysis pipeline.
#[allow(dead_code)]
pub fn model_supports_vision(base_url: &str, model_name: &str) -> bool {
    get_model_capabilities(base_url, model_name).map(|c| c.vision).unwrap_or(false)
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
}

/// Generate structured JSON output from a local model via Ollama's
/// `/api/generate` endpoint with `format: json`.
///
/// Authority: this is a candidate generator only. The returned JSON must be
/// validated and deterministically rule-matched downstream (Phase 5).
pub fn generate_structured(
    base_url: &str,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    let url = if base_url.is_empty() {
        DEFAULT_OLLAMA_URL.to_string()
    } else {
        base_url.to_string()
    };
    let api_url = format!("{}/api/generate", url);

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "format": "json",
        "stream": false,
        "options": { "temperature": 0.0 }
    });

    let output = StdCommand::new("curl")
        .args([
            "-s",
            "--connect-timeout",
            "60",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            &body.to_string(),
            &api_url,
        ])
        .output()
        .map_err(|e| format!("Failed to call Ollama generate: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Ollama generate failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: GenerateResponse = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse generate response: {}", e))?;

    Ok(parsed.response)
}

