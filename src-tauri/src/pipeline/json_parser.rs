use serde::Deserialize;
use std::collections::HashMap;
use std::io::Read;

/// A single conversation exported from ChatGPT.
/// All fields that can be null or absent in conversations.json are `Option<T>`.
#[derive(Debug, Deserialize)]
pub struct ConversationExport {
    pub id: String,
    pub title: Option<String>,
    pub create_time: Option<f64>,
    pub update_time: Option<f64>,
    #[serde(default)]
    pub mapping: HashMap<String, MessageNode>,
    pub current_node: Option<String>,
}

/// A node in the conversation tree (each message is a node with parent/children links).
#[derive(Debug, Deserialize, Clone)]
pub struct MessageNode {
    pub id: String,
    pub parent: Option<String>,
    #[serde(default)]
    pub children: Vec<String>,
    pub message: Option<Message>,
}

/// An individual message within a node.
#[derive(Debug, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub author: Author,
    pub create_time: Option<f64>,
    pub content: Option<Content>,
    /// Absorb all metadata shapes — do NOT define a strict struct here.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// The role and optional name of a message author.
#[derive(Debug, Deserialize, Clone)]
pub struct Author {
    pub role: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// Message content with a type discriminator and mixed-type parts.
#[derive(Debug, Deserialize, Clone)]
pub struct Content {
    pub content_type: String,
    /// parts can be strings, objects, or mixed — use Value to absorb all formats
    /// (ChatGPT schema changed in March 2025; Value handles both old and new layouts).
    #[serde(default)]
    pub parts: Vec<serde_json::Value>,
}

/// Streams `ConversationExport` elements from a reader containing a top-level JSON array.
///
/// CRITICAL: This uses `Deserializer::from_reader().into_iter()` — NOT `StreamDeserializer`.
/// `StreamDeserializer` handles multiple top-level values (NDJSON).
/// `into_iter()` handles a single top-level array, yielding one element at a time through
/// serde's sequence visitor — this is the correct pattern for conversations.json.
pub fn stream_conversations<R: Read>(
    reader: R,
) -> impl Iterator<Item = Result<ConversationExport, serde_json::Error>> {
    serde_json::Deserializer::from_reader(reader).into_iter::<ConversationExport>()
}
