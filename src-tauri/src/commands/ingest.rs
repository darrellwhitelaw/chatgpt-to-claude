use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum IngestEvent {
    Started,
    ExtractingZip,
    ParsingConversations { processed: u32 },
    BuildingIndex,
    Complete { total: u32, earliest_year: i32, latest_year: i32 },
    Error { message: String },
}

#[tauri::command]
pub async fn parse_zip(
    _path: String,
    on_event: Channel<IngestEvent>,
) -> Result<(), String> {
    // Stub — full implementation in plan 01-02
    on_event.send(IngestEvent::Started).map_err(|e| e.to_string())?;
    on_event
        .send(IngestEvent::Complete {
            total: 0,
            earliest_year: 2023,
            latest_year: 2023,
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}
