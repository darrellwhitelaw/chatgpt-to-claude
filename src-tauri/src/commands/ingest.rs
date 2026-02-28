use crate::pipeline::{json_parser, normalizer, zip_reader};
use crate::store::db;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::{ipc::Channel, State};

/// Events emitted over the IPC channel during ZIP ingestion.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum IngestEvent {
    Started,
    ExtractingZip,
    ParsingConversations { processed: u32 },
    BuildingIndex,
    Complete {
        total: u32,
        earliest_year: i32,
        latest_year: i32,
    },
    Error {
        message: String,
    },
}

/// Parses a ChatGPT export ZIP, streams all conversations into SQLite,
/// and emits typed progress events back to the frontend via Channel.
///
/// Pipeline:
///   1. Open ZIP, read conversations.json bytes into memory (Vec<u8>)
///   2. Stream-deserialize the JSON array element by element (no full-array load)
///   3. Normalize each ConversationExport to a ConversationRecord
///   4. Insert into SQLite using INSERT OR REPLACE (idempotent)
///   5. Emit Complete with total count and year range
#[tauri::command]
pub async fn parse_zip(
    path: String,
    on_event: Channel<IngestEvent>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    on_event
        .send(IngestEvent::Started)
        .map_err(|e| e.to_string())?;

    on_event
        .send(IngestEvent::ExtractingZip)
        .map_err(|e| e.to_string())?;

    let reader = zip_reader::open_conversations_entry(&path)?;

    on_event
        .send(IngestEvent::ParsingConversations { processed: 0 })
        .map_err(|e| e.to_string())?;

    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut count: u32 = 0;
    let mut earliest_year: i32 = i32::MAX;
    let mut latest_year: i32 = i32::MIN;

    for result in json_parser::stream_conversations(reader) {
        let export = match result {
            Ok(e) => e,
            Err(err) => {
                // Log and skip malformed entries — do not crash (IMP-05)
                eprintln!("Skipping malformed conversation: {err}");
                continue;
            }
        };

        // Track year range from create_time (Unix timestamp)
        if let Some(ts) = export.create_time {
            // Rough conversion: ts / seconds_per_year + 1970
            let year = (ts / 31_536_000.0) as i32 + 1970;
            if year < earliest_year {
                earliest_year = year;
            }
            if year > latest_year {
                latest_year = year;
            }
        }

        let record = normalizer::normalize(export);

        db::insert_conversation(&db, &record).map_err(|e| e.to_string())?;

        count += 1;

        // Emit progress every 50 conversations to avoid flooding the channel
        if count % 50 == 0 {
            on_event
                .send(IngestEvent::ParsingConversations { processed: count })
                .map_err(|e| e.to_string())?;
        }
    }

    on_event
        .send(IngestEvent::BuildingIndex)
        .map_err(|e| e.to_string())?;

    // Clamp year range if no timestamps found
    if earliest_year == i32::MAX {
        earliest_year = 2020;
    }
    if latest_year == i32::MIN {
        latest_year = 2024;
    }

    on_event
        .send(IngestEvent::Complete {
            total: count,
            earliest_year,
            latest_year,
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}
