use crate::pipeline::normalizer::ConversationRecord;
use rusqlite::{params, Connection, Result};

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(include_str!("schema.sql"))?;
    // Migration: add gizmo_id column for existing databases (safe to run multiple times)
    let _ = conn.execute("ALTER TABLE conversations ADD COLUMN gizmo_id TEXT", []);
    Ok(())
}

/// Inserts or replaces a conversation record in SQLite.
/// Uses INSERT OR REPLACE for idempotent re-runs (re-importing the same ZIP is safe).
pub fn insert_conversation(conn: &Connection, record: &ConversationRecord) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO conversations
            (id, title, created_at, message_count, has_images, has_code, token_estimate, full_text, gizmo_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            record.id,
            record.title,
            record.created_at,
            record.message_count,
            record.has_images as i32,
            record.has_code as i32,
            record.token_estimate,
            record.full_text,
            record.gizmo_id,
        ],
    )?;
    Ok(())
}

/// Writes Phase 2 clustering results back to a conversation row.
pub fn update_cluster_result(
    conn: &Connection,
    conversation_id: &str,
    cluster_label: &str,
    summary: &str,
    instructions: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE conversations SET cluster_label = ?1, summary = ?2, instructions = ?3 WHERE id = ?4",
        params![cluster_label, summary, instructions, conversation_id],
    )?;
    Ok(())
}

pub struct ConversationRow {
    pub id: String,
    pub title: Option<String>,
    pub full_text: String,
    pub token_estimate: i64,
}

/// Fetches all conversations ordered by creation time for batch clustering.
pub fn get_all_conversations(conn: &Connection) -> Result<Vec<ConversationRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, full_text, token_estimate FROM conversations ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ConversationRow {
            id: row.get(0)?,
            title: row.get(1)?,
            full_text: row.get(2)?,
            token_estimate: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub struct ExportRow {
    pub id: String,
    pub title: Option<String>,
    pub created_at: Option<i64>,
    pub full_text: String,
    pub gizmo_id: Option<String>,
}

/// Fetches all conversations for markdown export.
pub fn get_conversations_for_export(conn: &Connection) -> Result<Vec<ExportRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, created_at, full_text, gizmo_id FROM conversations ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ExportRow {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
            full_text: row.get(3)?,
            gizmo_id: row.get(4)?,
        })
    })?;
    rows.collect()
}
