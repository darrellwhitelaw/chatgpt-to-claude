// Stub — full implementation in Task 2 of plan 01-02
use crate::pipeline::json_parser::ConversationExport;

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

pub fn normalize(_export: ConversationExport) -> ConversationRecord {
    unimplemented!("Task 2 will implement this")
}
