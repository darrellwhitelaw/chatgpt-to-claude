use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const MODEL: &str = "claude-3-5-haiku-20241022";
pub const ANTHROPIC_VERSION: &str = "2023-06-01";
pub const BATCH_API_URL: &str = "https://api.anthropic.com/v1/messages/batches";
pub const MESSAGES_API_URL: &str = "https://api.anthropic.com/v1/messages";

#[derive(Serialize)]
pub struct BatchRequestItem {
    pub custom_id: String,
    pub params: BatchParams,
}

#[derive(Serialize)]
pub struct BatchParams {
    pub model: String,
    pub max_tokens: u32,
    pub system: String,
    pub messages: Vec<Message>,
}

#[derive(Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct BatchResult {
    pub id: String,
    pub processing_status: String,
    pub results_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BatchResultItem {
    pub custom_id: String,
    pub result: BatchResultContent,
}

#[derive(Deserialize, Debug)]
pub struct BatchResultContent {
    #[serde(rename = "type")]
    pub result_type: String,
    pub message: Option<BatchMessage>,
    pub error: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct BatchMessage {
    pub content: Vec<ContentBlock>,
}

#[derive(Deserialize, Debug)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: Option<String>,
}

/// Submit a batch and return the batch ID
pub async fn create_batch(
    client: &Client,
    api_key: &str,
    requests: Vec<BatchRequestItem>,
) -> Result<String, String> {
    let response = client
        .post(BATCH_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .header("anthropic-beta", "message-batches-2024-09-24")
        .json(&serde_json::json!({ "requests": requests }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Batch create failed {}: {}", status, body));
    }

    let result: BatchResult = response.json().await.map_err(|e| e.to_string())?;
    Ok(result.id)
}

/// Poll batch status, return (is_complete, results_url)
pub async fn poll_batch(
    client: &Client,
    api_key: &str,
    batch_id: &str,
) -> Result<(bool, Option<String>), String> {
    let url = format!("{}/{}", BATCH_API_URL, batch_id);
    let response = client
        .get(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("anthropic-beta", "message-batches-2024-09-24")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Batch poll failed: {}", body));
    }

    let result: BatchResult = response.json().await.map_err(|e| e.to_string())?;
    let done = result.processing_status == "ended";
    Ok((done, result.results_url))
}

/// Fetch JSONL results, return HashMap<custom_id, (cluster_label, summary, instructions)>
pub async fn fetch_results(
    client: &Client,
    api_key: &str,
    results_url: &str,
) -> Result<HashMap<String, (String, String, Option<String>)>, String> {
    let response = client
        .get(results_url)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let text = response.text().await.map_err(|e| e.to_string())?;
    let mut map = HashMap::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }

        let item: BatchResultItem = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue, // skip malformed lines
        };

        if item.result.result_type != "succeeded" { continue; }

        let text_content = item.result.message
            .as_ref()
            .and_then(|m| m.content.first())
            .and_then(|b| b.text.as_deref())
            .unwrap_or("");

        // Parse the structured JSON response from the model
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text_content) {
            let cluster_label = parsed["cluster_label"].as_str().unwrap_or("Uncategorized").to_string();
            let summary = parsed["summary"].as_str().unwrap_or("").to_string();
            let instructions = parsed["instructions"].as_str().map(|s| s.to_string());
            map.insert(item.custom_id, (cluster_label, summary, instructions));
        }
    }

    Ok(map)
}

/// Pass 1: single synchronous Messages API call to discover cluster vocabulary
pub async fn discover_clusters(
    client: &Client,
    api_key: &str,
    titles_and_snippets: &str,
) -> Result<Vec<String>, String> {
    use super::prompts;

    let response = client
        .post(MESSAGES_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "model": MODEL,
            "max_tokens": 512,
            "system": prompts::PASS1_SYSTEM_PROMPT,
            "messages": [{"role": "user", "content": prompts::build_pass1_message(titles_and_snippets)}]
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Pass 1 failed: {}", body));
    }

    let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let text = body["content"][0]["text"].as_str().unwrap_or("{}");
    let parsed: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;

    let labels: Vec<String> = parsed["labels"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    if labels.is_empty() {
        return Err("Pass 1 returned no cluster labels".to_string());
    }

    Ok(labels)
}
