use std::io::BufReader;

/// Opens the ZIP at `path` and returns a BufReader over the conversations.json entry.
/// Tries `conversations.json` at root first; falls back to searching any entry whose
/// name ends with `/conversations.json` to handle sub-directory variants.
///
/// Note on the borrow issue: `ZipFile` borrows from `ZipArchive`, making it impossible
/// to return from the function that owns the archive. Reading to `Vec<u8>` then wrapping
/// in `Cursor` is the standard idiomatic resolution. The JSON streaming still works
/// element-by-element — memory usage is proportional to `conversations.json` size, not
/// the full ZIP (attachments are not loaded).
pub fn open_conversations_entry(path: &str) -> Result<ConversationsReader, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Cannot open ZIP: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP: {e}"))?;

    // Try root-level entry first
    let bytes = if archive.by_name("conversations.json").is_ok() {
        let mut entry = archive
            .by_name("conversations.json")
            .map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut buf)
            .map_err(|e| format!("Cannot read conversations.json: {e}"))?;
        buf
    } else {
        // Fallback: find at any depth
        let idx = (0..archive.len())
            .find(|&i| {
                archive
                    .by_index(i)
                    .map(|e| {
                        e.name() == "conversations.json"
                            || e.name().ends_with("/conversations.json")
                    })
                    .unwrap_or(false)
            })
            .ok_or_else(|| "conversations.json not found in ZIP".to_string())?;
        let mut entry = archive.by_index(idx).map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut buf)
            .map_err(|e| format!("Cannot read conversations.json: {e}"))?;
        buf
    };

    // Wrap bytes in BufReader<Cursor> so json_parser receives a standard BufRead
    Ok(BufReader::new(std::io::Cursor::new(bytes)))
}

/// Concrete reader type alias — BufReader over an in-memory Cursor.
/// Used by json_parser::stream_conversations.
pub type ConversationsReader = BufReader<std::io::Cursor<Vec<u8>>>;
