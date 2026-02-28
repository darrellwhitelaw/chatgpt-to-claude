use keyring::Entry;
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;
use crate::store::db;

const SERVICE: &str = "com.darrellwhitelaw.chatgpt-to-claude";
const USER: &str = "anthropic-api-key";
const MODEL: &str = "claude-haiku-3-5-20241022";
const CLUSTERING_SYSTEM_PROMPT: &str =
    "You are a conversation analyst. Analyze the following ChatGPT conversation \
     transcript and return a JSON object with fields: cluster_label (string), \
     summary (string, 3-5 sentences), instructions (string or null).";

// Full text truncation — prevent 256MB batch limit (Pitfall 4)
const MAX_FULL_TEXT_CHARS: usize = 8_000;

#[derive(Serialize, Deserialize)]
pub struct CostEstimate {
    pub input_tokens: u64,
    pub estimated_usd: f64,
}

#[tauri::command]
pub async fn estimate_cost(state: State<'_, AppState>) -> Result<CostEstimate, String> {
    // 1. Get API key from Keychain
    let entry = Entry::new(SERVICE, USER).map_err(|e| e.to_string())?;
    let api_key = entry.get_password().map_err(|e| e.to_string())?;

    // 2. Fetch all conversations
    let conversations = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::get_all_conversations(&conn).map_err(|e| e.to_string())?
    };

    let conversation_count = conversations.len() as u64;

    // 3. Concatenate full_text (truncated per conversation — prevents 256MB batch limit)
    let all_text: String = conversations
        .iter()
        .map(|c| {
            let text = &c.full_text;
            if text.len() > MAX_FULL_TEXT_CHARS {
                &text[..MAX_FULL_TEXT_CHARS]
            } else {
                text.as_str()
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");

    // 4. Call count_tokens endpoint
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages/count_tokens")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "model": MODEL,
            "system": CLUSTERING_SYSTEM_PROMPT,
            "messages": [{"role": "user", "content": all_text}]
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // Handle 401 — bad key: delete from Keychain, return special error
    if response.status() == 401 {
        let _ = entry.delete_credential(); // best-effort — don't fail if already gone
        return Err("INVALID_API_KEY: Invalid API key — check console.anthropic.com".to_string());
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, body));
    }

    let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let input_tokens = body["input_tokens"].as_u64().unwrap_or(0);

    // 5. Compute cost — haiku-3-5 batch pricing (source: RESEARCH.md)
    // input: $0.40/MTok, output: $2.00/MTok, estimated ~300 output tokens/conversation
    let input_cost = (input_tokens as f64 * 0.40) / 1_000_000.0;
    let output_cost = (conversation_count as f64 * 300.0 * 2.00) / 1_000_000.0;
    let estimated_usd = input_cost + output_cost;

    Ok(CostEstimate { input_tokens, estimated_usd })
}
