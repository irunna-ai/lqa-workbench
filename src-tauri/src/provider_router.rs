// Phase 13 + Alpha Polish: Configurable AI Providers / BYOK
// Provider abstraction with OpenAI-compatible adapter and secure credential storage.
use serde::{Serialize, Deserialize};
use crate::ai_provider;

/// Supported provider types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    Ollama,
    OpenAI,
    Gemini,
    OpenRouter,
    LMStudio,
    OpenAiCompatible,
}

impl ProviderType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ollama" => Some(Self::Ollama),
            "openai" => Some(Self::OpenAI),
            "gemini" => Some(Self::Gemini),
            "openrouter" => Some(Self::OpenRouter),
            "lmstudio" | "lm_studio" => Some(Self::LMStudio),
            "openai_compatible" | "custom" => Some(Self::OpenAiCompatible),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::OpenAI => "openai",
            Self::Gemini => "gemini",
            Self::OpenRouter => "openrouter",
            Self::LMStudio => "lmstudio",
            Self::OpenAiCompatible => "openai_compatible",
        }
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Ollama => "Ollama",
            Self::OpenAI => "OpenAI",
            Self::Gemini => "Google Gemini",
            Self::OpenRouter => "OpenRouter",
            Self::LMStudio => "LM Studio",
            Self::OpenAiCompatible => "Custom OpenAI Compatible",
        }
    }
    pub fn default_endpoint(&self) -> &'static str {
        match self {
            Self::Ollama => "http://localhost:11434",
            Self::OpenAI => "https://api.openai.com/v1",
            Self::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Self::OpenRouter => "https://openrouter.ai/api/v1",
            Self::LMStudio => "http://localhost:1234/v1",
            Self::OpenAiCompatible => "",
        }
    }
    pub fn requires_api_key(&self) -> bool {
        match self {
            Self::Ollama | Self::LMStudio => false,
            _ => true,
        }
    }
}

/// Provider configuration with credential reference (not plaintext).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: String,
    pub display_name: String,
    pub endpoint: String,
    pub model: String,
    pub capabilities_json: String,
    pub active: bool,
}

/// Store an API key securely using OS keyring.
pub fn store_api_key(service: &str, username: &str, key: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(service, username)
        .map_err(|e| format!("keyring access: {}", e))?;
    entry.set_password(key).map_err(|e| format!("store key: {}", e))?;
    Ok(())
}

/// Retrieve an API key from OS keyring.
pub fn get_api_key(service: &str, username: &str) -> Result<String, String> {
    let entry = keyring::Entry::new(service, username)
        .map_err(|e| format!("keyring access: {}", e))?;
    entry.get_password().map_err(|e| format!("get key: {}", e))
}

/// Delete an API key from OS keyring.
pub fn delete_api_key(service: &str, username: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(service, username)
        .map_err(|e| format!("keyring access: {}", e))?;
    entry.delete_credential().map_err(|e| format!("delete key: {}", e))?;
    Ok(())
}

/// Test connection result.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestConnectionResult {
    pub success: bool,
    pub latency_ms: u64,
    pub message: String,
    pub version: Option<String>,
}

/// Test connection to a provider.
pub fn test_connection(provider_type: &str, endpoint: &str, api_key_ref: &str) -> TestConnectionResult {
    let start = std::time::Instant::now();
    let result = match ProviderType::from_str(provider_type).unwrap_or(ProviderType::Ollama) {
        ProviderType::Ollama => {
            let health = ai_provider::check_health(endpoint);
            if health.healthy { Ok(Some(health.version)) } else { Err(health.error.unwrap_or_else(|| "Connection failed".to_string())) }
        }
        _ => test_openai_compatible(endpoint, api_key_ref),
    };
    let latency = start.elapsed().as_millis() as u64;
    match result {
        Ok(ver) => TestConnectionResult { success: true, latency_ms: latency, message: "Connected successfully".to_string(), version: ver },
        Err(e) => TestConnectionResult { success: false, latency_ms: latency, message: e, version: None },
    }
}

/// Test an OpenAI-compatible endpoint by listing models.
fn test_openai_compatible(endpoint: &str, api_key_ref: &str) -> Result<Option<String>, String> {
    let url = format!("{}/models", endpoint.trim_end_matches('/'));
    let client = reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(10)).build().map_err(|e| format!("HTTP client: {}", e))?;
    let api_key = if api_key_ref.is_empty() { String::new() } else { get_api_key("qaivra", api_key_ref).unwrap_or_default() };
    let mut req = client.get(&url);
    if !api_key.is_empty() { req = req.header("Authorization", format!("Bearer {}", api_key)); }
    let resp = req.send().map_err(|e| format!("Request failed: {}", e))?;
    let status = resp.status();
    if status.is_success() { Ok(None) } else {
        let text = resp.text().unwrap_or_default();
        Err(format!("HTTP {}: {}", status, &text[..text.len().min(200)]))
    }
}

/// List models from an OpenAI-compatible endpoint.
pub fn list_models_openai_compatible(endpoint: &str, api_key_ref: &str) -> Result<Vec<String>, String> {
    let url = format!("{}/models", endpoint.trim_end_matches('/'));
    let client = reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(10)).build().map_err(|e| format!("HTTP client: {}", e))?;
    let api_key = if api_key_ref.is_empty() { String::new() } else { get_api_key("qaivra", api_key_ref).unwrap_or_default() };
    let mut req = client.get(&url);
    if !api_key.is_empty() { req = req.header("Authorization", format!("Bearer {}", api_key)); }
    let resp = req.send().map_err(|e| format!("Request failed: {}", e))?;
    let status = resp.status();
    let text = resp.text().map_err(|e| format!("Read response: {}", e))?;
    if !status.is_success() { return Err(format!("HTTP {}: {}", status, &text[..text.len().min(200)])); }
    let parsed: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("Parse response: {}", e))?;
    Ok(parsed["data"].as_array().ok_or("Missing 'data' field")?.iter().filter_map(|m| m["id"].as_str().map(|s| s.to_string())).collect())
}

/// Generate structured JSON via the active provider.
pub fn generate_structured(provider_type: &str, endpoint: &str, model: &str, prompt: &str) -> Result<String, String> {
    match ProviderType::from_str(provider_type).unwrap_or(ProviderType::Ollama) {
        ProviderType::Ollama => ai_provider::generate_structured(endpoint, model, prompt),
        _ => generate_openai_compatible(endpoint, model, prompt),
    }
}

/// OpenAI-compatible chat completion via HTTP.
fn generate_openai_compatible(endpoint: &str, model: &str, prompt: &str) -> Result<String, String> {
    let api_key = get_api_key("qaivra", "openai_api_key").unwrap_or_default();
    let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": "You are a localization QA assistant. Always respond with valid JSON."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.0,
        "response_format": {"type": "json_object"}
    });

    let client = reqwest::blocking::Client::new();
    let mut req = client.post(&url)
        .header("Content-Type", "application/json")
        .json(&body);
    if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    let resp = req.send().map_err(|e| format!("OpenAI request failed: {}", e))?;
    let status = resp.status();
    let text = resp.text().map_err(|e| format!("read response: {}", e))?;

    if !status.is_success() {
        return Err(format!("OpenAI API error {}: {}", status, text));
    }

    let parsed: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("parse response: {}", e))?;
    let content = parsed["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Missing content in response")?;
    Ok(content.to_string())
}