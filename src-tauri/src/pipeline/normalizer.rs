use crate::pipeline::json_parser::{Content, ConversationExport};

/// Flat record ready for SQLite insert.
pub struct ConversationRecord {
    pub id: String,
    pub title: String,
    pub created_at: Option<i64>,
    pub message_count: u32,
    pub has_images: bool,
    pub has_code: bool,
    pub token_estimate: u32,
    pub full_text: String,
}

/// Normalizes a `ConversationExport` to a `ConversationRecord`.
///
/// Traversal (IMP-06) is NOT done here — normalizer works on the raw export.
/// Traversal is wired in the command after plan 01-03 lands.
/// For now: count messages from mapping values whose author role is user/assistant.
pub fn normalize(export: ConversationExport) -> ConversationRecord {
    let title = export
        .title
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Untitled".to_string());

    let created_at = export.create_time.map(|t| t as i64);

    let mut message_count: u32 = 0;
    let mut has_images = false;
    let mut has_code = false;
    let mut full_text = String::new();

    for node in export.mapping.values() {
        if let Some(ref msg) = node.message {
            let role = msg.author.role.as_str();
            if role == "user" || role == "assistant" {
                message_count += 1;
                if let Some(ref content) = msg.content {
                    let extracted = extract_text(content);
                    if !extracted.is_empty() {
                        full_text.push_str(&extracted);
                        full_text.push('\n');
                    }
                    if content.content_type == "multimodal_text" {
                        has_images = true;
                    }
                    if content.content_type == "code" {
                        has_code = true;
                    }
                }
            }
        }
    }

    // Rough token estimate: ~4 chars per token
    let token_estimate = (full_text.len() / 4) as u32;

    ConversationRecord {
        id: export.id,
        title,
        created_at,
        message_count,
        has_images,
        has_code,
        token_estimate,
        full_text,
    }
}

/// Extracts plain-text strings from Content parts.
/// Parts can be strings, objects, or null — only String variants are included.
fn extract_text(content: &Content) -> String {
    content
        .parts
        .iter()
        .filter_map(|part| part.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>()
        .join(" ")
}
