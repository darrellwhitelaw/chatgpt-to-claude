use tauri_app_lib::pipeline::json_parser::{Author, Content, Message, MessageNode};
use tauri_app_lib::pipeline::traversal::linearize_messages;
use std::collections::HashMap;

/// Helper: build a minimal MessageNode with user message
fn user_node(id: &str, parent: Option<&str>, text: &str) -> MessageNode {
    MessageNode {
        id: id.to_string(),
        parent: parent.map(|s| s.to_string()),
        children: vec![],
        message: Some(Message {
            id: id.to_string(),
            author: Author { role: "user".to_string(), name: None },
            create_time: None,
            content: Some(Content {
                content_type: "text".to_string(),
                parts: vec![serde_json::Value::String(text.to_string())],
            }),
            metadata: serde_json::Value::Null,
        }),
    }
}

/// Helper: build an assistant MessageNode
fn assistant_node(id: &str, parent: Option<&str>, text: &str) -> MessageNode {
    MessageNode {
        id: id.to_string(),
        parent: parent.map(|s| s.to_string()),
        children: vec![],
        message: Some(Message {
            id: id.to_string(),
            author: Author { role: "assistant".to_string(), name: None },
            create_time: None,
            content: Some(Content {
                content_type: "text".to_string(),
                parts: vec![serde_json::Value::String(text.to_string())],
            }),
            metadata: serde_json::Value::Null,
        }),
    }
}

/// Helper: build a structural node (no message — root or branch point)
fn struct_node(id: &str, parent: Option<&str>) -> MessageNode {
    MessageNode {
        id: id.to_string(),
        parent: parent.map(|s| s.to_string()),
        children: vec![],
        message: None,
    }
}

#[test]
fn test_linear_chain() {
    let mut mapping = HashMap::new();
    mapping.insert("root".to_string(), struct_node("root", None));
    mapping.insert("a".to_string(), user_node("a", Some("root"), "Hello"));
    mapping.insert("b".to_string(), assistant_node("b", Some("a"), "Hi there"));
    mapping.insert("c".to_string(), user_node("c", Some("b"), "Thanks"));

    let messages = linearize_messages(&mapping, "c");
    assert_eq!(messages.len(), 3, "Expected 3 messages in linear chain");
    // Verify chronological order (root -> leaf)
    assert_eq!(
        messages[0].author.role, "user",
        "First message should be user"
    );
    assert_eq!(
        messages[2].author.role, "user",
        "Last message should be user"
    );
}

#[test]
fn test_branched_conversation_follows_current_node() {
    // B1 and B2 are both children of A; current_node points to C (via B2)
    // B1 must NOT appear in output
    let mut mapping = HashMap::new();
    mapping.insert("root".to_string(), struct_node("root", None));
    mapping.insert("a".to_string(), user_node("a", Some("root"), "Hello"));
    mapping.insert("b1".to_string(), assistant_node("b1", Some("a"), "Response v1"));
    mapping.insert("b2".to_string(), assistant_node("b2", Some("a"), "Response v2"));
    mapping.insert("c".to_string(), user_node("c", Some("b2"), "Thanks"));

    let messages = linearize_messages(&mapping, "c");

    // Must include b2, must NOT include b1
    let has_v1 = messages
        .iter()
        .any(|m| {
            m.content
                .as_ref()
                .and_then(|c| c.parts.first())
                .and_then(|p| p.as_str())
                == Some("Response v1")
        });
    let has_v2 = messages
        .iter()
        .any(|m| {
            m.content
                .as_ref()
                .and_then(|c| c.parts.first())
                .and_then(|p| p.as_str())
                == Some("Response v2")
        });

    assert!(!has_v1, "Branched-off response v1 must NOT appear in output");
    assert!(has_v2, "Active branch response v2 MUST appear in output");
    assert_eq!(messages.len(), 3, "Should have user + v2 assistant + user");
}

#[test]
fn test_missing_parent_node_does_not_panic() {
    // current_node's parent chain leads to a node ID not in mapping
    let mut mapping = HashMap::new();
    mapping.insert("c".to_string(), user_node("c", Some("missing_parent"), "Hello"));

    // Must not panic — returns partial chain
    let messages = linearize_messages(&mapping, "c");
    assert!(messages.len() <= 1, "Should return at most the node itself");
}

#[test]
fn test_empty_mapping_returns_empty() {
    let mapping: HashMap<String, MessageNode> = HashMap::new();
    let messages = linearize_messages(&mapping, "any_node");
    assert!(messages.is_empty(), "Empty mapping should return empty vec");
}

#[test]
fn test_system_messages_excluded() {
    let mut mapping = HashMap::new();
    mapping.insert("root".to_string(), struct_node("root", None));
    // System message — should be filtered
    mapping.insert(
        "sys".to_string(),
        MessageNode {
            id: "sys".to_string(),
            parent: Some("root".to_string()),
            children: vec![],
            message: Some(Message {
                id: "sys".to_string(),
                author: Author { role: "system".to_string(), name: None },
                create_time: None,
                content: Some(Content {
                    content_type: "text".to_string(),
                    parts: vec![serde_json::Value::String("System init".to_string())],
                }),
                metadata: serde_json::Value::Null,
            }),
        },
    );
    mapping.insert("a".to_string(), user_node("a", Some("sys"), "Hello"));

    let messages = linearize_messages(&mapping, "a");
    assert_eq!(messages.len(), 1, "System message must be excluded");
    assert_eq!(messages[0].author.role, "user");
}
