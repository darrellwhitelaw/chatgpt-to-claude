use std::io::BufReader;

/// Opens the ZIP at `path` and returns a BufReader over a merged conversations JSON array.
///
/// Handles two ChatGPT export formats:
///
/// - **Legacy** (pre-2026): single `conversations.json` at root or any subdirectory.
/// - **Sharded** (2026+): OpenAI split exports into `conversations-000.json`,
///   `conversations-001.json`, … `conversations-NNN.json`. Each shard is a
///   top-level JSON array. We merge all shards into one flat array so that the
///   downstream `json_parser::stream_conversations` receives a single `[…]`
///   without any other changes to the pipeline.
///
/// `__MACOSX/._*` Apple double-entry files are skipped automatically.
pub fn open_conversations_entry(path: &str) -> Result<ConversationsReader, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Cannot open ZIP: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP: {e}"))?;

    // --- Step 1: collect all matching entry names, sorted for deterministic order ---
    let mut entry_names: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            archive.by_index(i).ok().and_then(|e| {
                let name = e.name().to_string();
                // Skip macOS metadata entries
                if name.contains("/__MACOSX/") || name.starts_with("__MACOSX/") {
                    return None;
                }
                let basename = name.rsplit('/').next().unwrap_or(&name);
                if basename.starts_with("._") {
                    return None;
                }
                // Match legacy single file or new sharded files
                let is_match = basename == "conversations.json"
                    || (basename.starts_with("conversations-") && basename.ends_with(".json"));
                if is_match { Some(name) } else { None }
            })
        })
        .collect();

    if entry_names.is_empty() {
        return Err("conversations.json not found in ZIP".to_string());
    }

    // Sort so shards are merged in order: conversations-000, 001, 002, …
    // The legacy conversations.json will sort before conversations-NNN.json
    entry_names.sort();

    // --- Step 2: read each entry into memory ---
    let mut chunks: Vec<Vec<u8>> = Vec::with_capacity(entry_names.len());
    for name in &entry_names {
        let mut entry = archive.by_name(name).map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut buf)
            .map_err(|e| format!("Cannot read {name}: {e}"))?;
        chunks.push(buf);
    }

    // --- Step 3: merge into one flat JSON array ---
    // Each chunk is a JSON array `[…]`. Strip the outer brackets from each,
    // join the inner content with commas, then wrap in a single `[…]`.
    let bytes = if chunks.len() == 1 {
        // Single file: return as-is, no merge needed
        chunks.remove(0)
    } else {
        let total_capacity: usize = chunks.iter().map(|b| b.len()).sum::<usize>() + chunks.len();
        let mut merged: Vec<u8> = Vec::with_capacity(total_capacity);
        merged.push(b'[');
        let mut first_non_empty = true;

        for chunk in &chunks {
            // Find the outer opening '[' and extract content after it
            let Some(open) = chunk.iter().position(|&b| b == b'[') else {
                continue;
            };
            let inner = &chunk[open + 1..];

            // Find the outer closing ']' (search from the right)
            let Some(close) = inner.iter().rposition(|&b| b == b']') else {
                continue;
            };
            let inner = &inner[..close];

            // Skip shards that contain only whitespace (empty arrays)
            if inner.iter().all(|b| b.is_ascii_whitespace()) {
                continue;
            }

            if !first_non_empty {
                merged.push(b',');
            }
            merged.extend_from_slice(inner);
            first_non_empty = false;
        }

        merged.push(b']');
        merged
    };

    Ok(BufReader::new(std::io::Cursor::new(bytes)))
}

/// Concrete reader type alias — BufReader over an in-memory Cursor.
/// Used by json_parser::stream_conversations.
pub type ConversationsReader = BufReader<std::io::Cursor<Vec<u8>>>;
