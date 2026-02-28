use crate::pipeline::json_parser::{Content, ConversationExport};
use crate::pipeline::traversal::linearize_messages;

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
/// Uses `linearize_messages` (IMP-06) to walk current_node → parent chain for
/// correct chronological order and branch selection — not mapping.values() iteration
/// which would produce wrong order and include all branches.
pub fn normalize(export: ConversationExport) -> ConversationRecord {
    let title = export
        .title
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Untitled".to_string());

    let created_at = export.create_time.map(|t| t as i64);

    let messages = if let Some(ref current_node) = export.current_node {
        linearize_messages(&export.mapping, current_node)
    } else {
        vec![] // No current_node — skip traversal, store empty
    };

    let message_count = messages.len() as u32;
    let mut has_images = false;
    let mut has_code = false;
    let mut full_text = String::new();

    for msg in &messages {
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
