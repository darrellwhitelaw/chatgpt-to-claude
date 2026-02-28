use crate::pipeline::json_parser::{Message, MessageNode};
use std::collections::HashMap;

/// Reconstructs the conversation's message order by walking the node-graph
/// from `current_node` backward through parent references, then reversing.
///
/// This is the ONLY correct way to traverse conversations.json:
/// - mapping.values() iteration produces random order (not conversation order)
/// - mapping.values() includes all branches (not just the branch the user saw)
///
/// The walk: current_node -> parent -> parent -> ... -> root (null parent)
/// Collected in reverse chronological order, then reversed to chronological.
pub fn linearize_messages(
    mapping: &HashMap<String, MessageNode>,
    current_node: &str,
) -> Vec<Message> {
    let mut messages: Vec<Message> = Vec::new();
    let mut node_id = Some(current_node.to_string());

    loop {
        let id = match node_id.take() {
            Some(id) => id,
            None => break,
        };

        let node = match mapping.get(&id) {
            Some(n) => n,
            None => break, // Missing node — terminate gracefully (IMP-05)
        };

        if let Some(ref msg) = node.message {
            if should_include_message(msg) {
                messages.push(msg.clone());
            }
        }

        node_id = node.parent.clone();
    }

    // Built leaf-to-root (leaf first); reverse to get chronological order
    messages.reverse();
    messages
}

/// Returns true if a message should be included in the linearized output.
/// Only user and assistant messages with non-empty content are included.
/// System, tool, memory, and tether_browsing messages are excluded.
pub fn should_include_message(msg: &Message) -> bool {
    let role = msg.author.role.as_str();
    if role != "user" && role != "assistant" {
        return false;
    }
    let Some(ref content) = msg.content else {
        return false;
    };
    // Exclude empty parts arrays
    if content.parts.is_empty() {
        return false;
    }
    // Exclude messages where all parts are null or empty string
    content.parts.iter().any(|p| {
        p.as_str().map(|s| !s.is_empty()).unwrap_or(
            // Non-string parts (image objects, etc.) count as non-empty
            !p.is_null(),
        )
    })
}
