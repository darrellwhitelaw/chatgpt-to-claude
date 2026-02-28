use keyring::Entry;
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri::ipc::Channel;
use std::time::Duration;
use crate::AppState;
use crate::store::db;
use crate::ai::{batch, prompts};

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

#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum ClusterEvent {
    EstimatingTokens,
    Pass1Started,
    Pass1Complete { cluster_labels: Vec<String> },
    BatchSubmitted { batch_id: String },
    Polling { elapsed_secs: u64 },
    Complete { assigned_count: usize },
    Error { message: String },
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

#[tauri::command]
pub async fn start_clustering(
    state: State<'_, AppState>,
    on_event: Channel<ClusterEvent>,
) -> Result<(), String> {
    // 1. Get API key from Keychain
    let entry = Entry::new(SERVICE, USER).map_err(|e| e.to_string())?;
    let api_key = entry.get_password().map_err(|e| e.to_string())?;

    // 2. Load all conversations from SQLite
    let conversations = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::get_all_conversations(&conn).map_err(|e| e.to_string())?
    };

    if conversations.is_empty() {
        return Err("No conversations found in database".to_string());
    }

    let client = reqwest::Client::new();

    // 3. Pass 1: build titles+snippets sample and discover cluster vocabulary
    let _ = on_event.send(ClusterEvent::Pass1Started);

    let titles_snippets: String = conversations
        .iter()
        .take(200) // sample up to 200 titles — sufficient for vocabulary discovery
        .map(|c| {
            let snippet: String = c.full_text.chars().take(200).collect();
            format!("Title: {}\nSnippet: {}", c.title.as_deref().unwrap_or("Untitled"), snippet)
        })
        .collect::<Vec<_>>()
        .join("\n---\n");

    let cluster_labels = batch::discover_clusters(&client, &api_key, &titles_snippets)
        .await
        .map_err(|e| {
            let _ = on_event.send(ClusterEvent::Error { message: e.clone() });
            e
        })?;

    let _ = on_event.send(ClusterEvent::Pass1Complete {
        cluster_labels: cluster_labels.clone(),
    });

    // 4. Pass 2: build batch requests with vocabulary embedded in system prompt
    let pass2_system = prompts::build_pass2_system(&cluster_labels);

    let requests: Vec<batch::BatchRequestItem> = conversations
        .iter()
        .map(|c| batch::BatchRequestItem {
            custom_id: c.id.clone(),
            params: batch::BatchParams {
                model: batch::MODEL.to_string(),
                max_tokens: 512,
                system: pass2_system.clone(),
                messages: vec![batch::Message {
                    role: "user".to_string(),
                    content: prompts::build_pass2_user_message(&c.full_text),
                }],
            },
        })
        .collect();

    // 5. Submit batch
    let batch_id = batch::create_batch(&client, &api_key, requests)
        .await
        .map_err(|e| {
            let _ = on_event.send(ClusterEvent::Error { message: e.clone() });
            e
        })?;

    let _ = on_event.send(ClusterEvent::BatchSubmitted { batch_id: batch_id.clone() });

    // 6. Poll loop — 5-second interval (user decision from CONTEXT.md)
    // NOTE: start_clustering is an async fn Tauri command running inside Tauri's tokio runtime.
    // tokio::time::sleep is safe here — the Pitfall 2 panic applies to tokio::spawn called
    // from outside a reactor context, not to awaiting within an existing async command.
    let start = std::time::Instant::now();
    let max_polls = 720; // 1 hour max (720 * 5s)
    let mut poll_count = 0;

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        poll_count += 1;

        let elapsed = start.elapsed().as_secs();
        let _ = on_event.send(ClusterEvent::Polling { elapsed_secs: elapsed });

        let (done, results_url) = batch::poll_batch(&client, &api_key, &batch_id)
            .await
            .map_err(|e| {
                let _ = on_event.send(ClusterEvent::Error { message: e.clone() });
                e.clone()
            })?;

        if done {
            let results_url = results_url.ok_or_else(|| {
                "Batch ended with no results_url".to_string()
            })?;

            // 7. Fetch and parse JSONL results (Pitfall 5: use custom_id, not position)
            let results = batch::fetch_results(&client, &api_key, &results_url)
                .await
                .map_err(|e| {
                    let _ = on_event.send(ClusterEvent::Error { message: e.clone() });
                    e
                })?;

            // 8. Write to SQLite
            let assigned_count = results.len();
            {
                let conn = state.db.lock().map_err(|e| e.to_string())?;
                for (conv_id, (cluster_label, summary, instructions)) in &results {
                    let _ = db::update_cluster_result(
                        &conn,
                        conv_id,
                        cluster_label,
                        summary,
                        instructions.as_deref(),
                    );
                }
            }

            let _ = on_event.send(ClusterEvent::Complete { assigned_count });
            return Ok(());
        }

        if poll_count >= max_polls {
            let err = "Batch timed out after 1 hour".to_string();
            let _ = on_event.send(ClusterEvent::Error { message: err.clone() });
            return Err(err);
        }
    }
}
