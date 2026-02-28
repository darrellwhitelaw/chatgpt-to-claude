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

/// Deserializes all `ConversationExport` elements from a reader containing a top-level
/// JSON array (the format OpenAI uses in conversations.json).
///
/// Note: `serde_json::Deserializer::into_iter()` is a StreamDeserializer for NDJSON
/// (multiple separate top-level values). It does NOT iterate array elements — it tries
/// to deserialize the whole `[...]` as one `ConversationExport`, fails, and returns 0.
/// The correct approach for a single top-level array is `from_reader::<Vec<T>>`.
/// conversations.json bytes are already in-memory (Vec<u8> from zip_reader), so there
/// is no additional memory cost from collecting into Vec<ConversationExport>.
pub fn stream_conversations<R: Read>(
    reader: R,
) -> Result<Vec<ConversationExport>, serde_json::Error> {
    serde_json::from_reader(reader)
}
