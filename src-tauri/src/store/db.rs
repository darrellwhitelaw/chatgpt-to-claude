use crate::pipeline::normalizer::ConversationRecord;
use rusqlite::{params, Connection, Result};

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(include_str!("schema.sql"))
}

/// Inserts or replaces a conversation record in SQLite.
/// Uses INSERT OR REPLACE for idempotent re-runs (re-importing the same ZIP is safe).
pub fn insert_conversation(conn: &Connection, record: &ConversationRecord) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO conversations
            (id, title, created_at, message_count, has_images, has_code, token_estimate, full_text)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            record.id,
            record.title,
            record.created_at,
            record.message_count,
            record.has_images as i32,
            record.has_code as i32,
            record.token_estimate,
            record.full_text,
        ],
    )?;
    Ok(())
}
