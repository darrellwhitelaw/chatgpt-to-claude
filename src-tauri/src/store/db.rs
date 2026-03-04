use crate::pipeline::normalizer::ConversationRecord;
use rusqlite::{params, Connection, Result};

pub fn init_schema(conn: &Connection) -> Result<()> {
    // WAL mode allows concurrent reads during background writes (e.g. clustering).
    // busy_timeout prevents "database is locked" on contention.
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA busy_timeout = 5000;
         PRAGMA foreign_keys = ON;
         PRAGMA cache_size = -2000;",
    )?;

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

pub struct MemoryCluster {
    pub cluster_label: String,
    pub count: i64,
    pub titles: Vec<String>,
    pub summary: Option<String>,
    pub instructions: Option<String>,
    pub earliest: Option<i64>,
    pub latest: Option<i64>,
}

/// Fetches cluster data grouped by cluster_label for Claude Code memory generation.
pub fn get_clusters_for_memory(conn: &Connection) -> Result<Vec<MemoryCluster>> {
    let mut stmt = conn.prepare(
        "SELECT cluster_label, COUNT(*) as cnt,
                GROUP_CONCAT(title, '|||') as titles,
                MIN(summary) as summary,
                MIN(instructions) as instructions,
                MIN(created_at) as earliest,
                MAX(created_at) as latest
         FROM conversations
         WHERE cluster_label IS NOT NULL AND cluster_label != ''
         GROUP BY cluster_label
         ORDER BY cnt DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let titles_str: String = row.get(2)?;
        let titles: Vec<String> = titles_str.split("|||").map(|s| s.to_string()).collect();
        Ok(MemoryCluster {
            cluster_label: row.get(0)?,
            count: row.get(1)?,
            titles,
            summary: row.get(3)?,
            instructions: row.get(4)?,
            earliest: row.get(5)?,
            latest: row.get(6)?,
        })
    })?;
    rows.collect()
}

/// Returns true if any conversations have cluster_label set (i.e., AI analysis was run).
pub fn has_cluster_data(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM conversations WHERE cluster_label IS NOT NULL AND cluster_label != ''",
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|c| c > 0)
    .unwrap_or(false)
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
