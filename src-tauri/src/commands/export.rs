use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;
use crate::AppState;
use crate::store::db;

#[derive(Serialize)]
pub struct ExportResult {
    pub files_written: usize,
    pub folder_path: String,
    pub mcp_configured: bool,
    pub media_extracted: usize,
}

/// Exports all conversations as markdown files to ~/Documents/ChatGPT History/
/// organized into year-based folders, then extracts all media files, group chats,
/// and supplementary data from the original ZIP, auto-configures the Claude Desktop
/// MCP filesystem server, and generates a README.md with instructions for Claude.
///
/// Structure:
///   ~/Documents/ChatGPT History/
///     README.md          ← instructions + context for Claude
///     2023/ 2024/ 2025/  ← conversations by year
///     group-chats/       ← group_chats.json exported as markdown
///     media/             ← all images from the ZIP
///     data/              ← shared_conversations.json
#[tauri::command]
pub async fn export_conversations(
    state: State<'_, AppState>,
) -> Result<ExportResult, String> {
    let conversations = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::get_conversations_for_export(&conn).map_err(|e| e.to_string())?
    };

    if conversations.is_empty() {
        return Err("No conversations found in database".to_string());
    }

    let zip_path: Option<String> = {
        let zp = state.zip_path.lock().map_err(|e| e.to_string())?;
        zp.clone()
    };

    let home = std::env::var("HOME").map_err(|_| "Cannot determine home directory".to_string())?;
    let root = PathBuf::from(&home).join("Documents").join("ChatGPT History");
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;

    // Remove legacy Projects/ folder if present from an older export
    let projects_dir = root.join("Projects");
    if projects_dir.exists() {
        let _ = std::fs::remove_dir_all(&projects_dir);
    }

    // Write conversations into year-based folders
    let mut files_written = 0;
    let mut seen: HashMap<String, usize> = HashMap::new();

    for conv in &conversations {
        let year = conv.created_at
            .map(|ts| unix_to_year(ts))
            .unwrap_or_else(|| "Unknown".to_string());

        let subfolder = root.join(&year);
        std::fs::create_dir_all(&subfolder).map_err(|e| e.to_string())?;

        let title = conv.title.as_deref().unwrap_or("Untitled");
        let base_slug = slugify(title);
        let key = format!("{}/{}", subfolder.to_string_lossy(), base_slug);
        let count = seen.entry(key).or_insert(0);
        let file_name = if *count == 0 {
            format!("{}.md", base_slug)
        } else {
            format!("{}-{}.md", base_slug, count)
        };
        *count += 1;

        let date_line = conv.created_at
            .map(|ts| format!("_{}_\n\n", unix_to_date_str(ts)))
            .unwrap_or_default();

        let content = format!(
            "# {}\n\n{}---\n\n{}\n",
            title,
            date_line,
            conv.full_text.trim()
        );

        std::fs::write(subfolder.join(&file_name), content)
            .map_err(|e| e.to_string())?;

        files_written += 1;
    }

    // Extract all assets from the original ZIP
    let mut media_extracted: usize = 0;
    let mut group_chats_written: usize = 0;
    let mut user_name: Option<String> = None;

    if let Some(ref zp) = zip_path {
        user_name = read_user_name(zp);
        media_extracted = extract_media(zp, &root).unwrap_or(0);
        group_chats_written = export_group_chats(zp, &root).unwrap_or(0);
        let _ = copy_shared_conversations(zp, &root);
    }

    // Generate README.md once — preserved if user edits it
    let readme_path = root.join("README.md");
    if !readme_path.exists() {
        let _ = generate_readme(
            &root,
            &conversations,
            user_name.as_deref(),
            media_extracted > 0,
            group_chats_written > 0,
        );
    }

    // Auto-configure Claude Desktop MCP filesystem server
    let export_path = root.to_string_lossy().to_string();
    let mcp_configured = update_claude_desktop_config(&home, &export_path).is_ok();

    Ok(ExportResult {
        files_written,
        folder_path: export_path,
        mcp_configured,
        media_extracted: media_extracted + group_chats_written,
    })
}

// ── Asset extraction ───────────────────────────────────────────────────────────

/// Extracts all image/media files from the ZIP into ~/Documents/ChatGPT History/media/
fn extract_media(zip_path: &str, root: &PathBuf) -> Result<usize, String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    let media_dir = root.join("media");
    std::fs::create_dir_all(&media_dir).map_err(|e| e.to_string())?;

    let mut extracted = 0;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;

        if entry.is_dir() || entry.name().contains("__MACOSX") {
            continue;
        }

        let filename = std::path::Path::new(entry.name())
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("")
            .to_string();

        if filename.is_empty() {
            continue;
        }

        let ext = std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" => {
                let mut buf = Vec::new();
                std::io::Read::read_to_end(&mut entry, &mut buf)
                    .map_err(|e| e.to_string())?;
                std::fs::write(media_dir.join(&filename), buf)
                    .map_err(|e| e.to_string())?;
                extracted += 1;
            }
            _ => {}
        }
    }

    Ok(extracted)
}

// ── Group chats export ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GroupChatsExport {
    chats: Vec<GroupChat>,
}

#[derive(Deserialize)]
struct GroupChat {
    name: String,
    created_at: Option<String>,
    messages: Vec<GroupChatMessage>,
}

#[derive(Deserialize)]
struct GroupChatMessage {
    role: String,
    text: String,
}

/// Exports group_chats.json as markdown files into ~/Documents/ChatGPT History/group-chats/
fn export_group_chats(zip_path: &str, root: &PathBuf) -> Result<usize, String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    // Find group_chats.json
    let bytes = {
        let mut found_idx = None;
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                if entry.name().ends_with("group_chats.json") {
                    found_idx = Some(i);
                    break;
                }
            }
        }
        match found_idx {
            None => return Ok(0),
            Some(idx) => {
                let mut entry = archive.by_index(idx).map_err(|e| e.to_string())?;
                let mut buf = Vec::new();
                std::io::Read::read_to_end(&mut entry, &mut buf)
                    .map_err(|e| e.to_string())?;
                buf
            }
        }
    };

    let export: GroupChatsExport = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return Ok(0),
    };

    if export.chats.is_empty() {
        return Ok(0);
    }

    let dir = root.join("group-chats");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let mut written = 0;
    let mut seen: HashMap<String, usize> = HashMap::new();

    for chat in &export.chats {
        let title = if chat.name.is_empty() { "Untitled Group Chat" } else { &chat.name };
        let base_slug = slugify(title);
        let count = seen.entry(base_slug.clone()).or_insert(0);
        let file_name = if *count == 0 {
            format!("{}.md", base_slug)
        } else {
            format!("{}-{}.md", base_slug, count)
        };
        *count += 1;

        // Take only YYYY-MM-DD from the ISO timestamp
        let date_line = chat.created_at.as_deref()
            .map(|d| format!("_{}_\n\n", &d[..d.len().min(10)]))
            .unwrap_or_default();

        let mut body = String::new();
        for msg in &chat.messages {
            if msg.text.is_empty() {
                continue;
            }
            let role_label = match msg.role.as_str() {
                "user" => "**You**",
                "assistant" => "**ChatGPT**",
                other => other,
            };
            body.push_str(&format!("{}: {}\n\n", role_label, msg.text.trim()));
        }

        let content = format!("# {}\n\n{}---\n\n{}\n", title, date_line, body.trim());
        std::fs::write(dir.join(&file_name), content).map_err(|e| e.to_string())?;
        written += 1;
    }

    Ok(written)
}

// ── Supplementary data files ───────────────────────────────────────────────────

/// Copies shared_conversations.json to ~/Documents/ChatGPT History/data/
fn copy_shared_conversations(zip_path: &str, root: &PathBuf) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let name = {
            let e = archive.by_index(i).map_err(|e| e.to_string())?;
            e.name().to_string()
        };
        if name.ends_with("shared_conversations.json") {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut buf).map_err(|e| e.to_string())?;
            if buf.len() <= 4 {
                return Ok(());
            }
            let data_dir = root.join("data");
            std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
            std::fs::write(data_dir.join("shared_conversations.json"), buf)
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    Ok(())
}

/// Reads the user's name from user.json in the ZIP (does not write PII to disk)
fn read_user_name(zip_path: &str) -> Option<String> {
    let file = std::fs::File::open(zip_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    for i in 0..archive.len() {
        let name = archive.by_index(i).ok()?.name().to_string();
        if name.ends_with("user.json") {
            let mut entry = archive.by_index(i).ok()?;
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut buf).ok()?;
            let json: serde_json::Value = serde_json::from_slice(&buf).ok()?;
            return json.get("name")?.as_str().map(String::from);
        }
    }
    None
}

// ── README.md generation ──────────────────────────────────────────────────────

fn generate_readme(
    root: &PathBuf,
    conversations: &[db::ExportRow],
    user_name: Option<&str>,
    has_media: bool,
    has_group_chats: bool,
) -> Result<(), String> {
    let all_titles: Vec<String> = conversations.iter()
        .filter_map(|c| c.title.clone())
        .collect();
    let top_topics = extract_top_topics(&all_titles, 15);

    let years: Vec<i32> = conversations.iter()
        .filter_map(|c| c.created_at)
        .filter_map(|ts| unix_to_year(ts).parse::<i32>().ok())
        .collect();
    let (earliest, latest) = years.iter().fold(
        (i32::MAX, i32::MIN),
        |(lo, hi), &y| (lo.min(y), hi.max(y)),
    );
    let year_range = if earliest == i32::MAX {
        "Unknown".to_string()
    } else if earliest == latest {
        earliest.to_string()
    } else {
        format!("{}–{}", earliest, latest)
    };

    let topics_list = if top_topics.is_empty() {
        "_(none detected)_".to_string()
    } else {
        top_topics.iter().map(|t| format!("- {}", t)).collect::<Vec<_>>().join("\n")
    };

    let name_line = user_name
        .map(|n| format!("- **Name:** {}", n))
        .unwrap_or_else(|| "- **Name:**".to_string());

    let mut extra_folders = String::new();
    if has_group_chats {
        extra_folders.push_str("  group-chats/       ← group conversations as markdown\n");
    }
    if has_media {
        extra_folders.push_str("  media/             ← images (uploads & DALL-E generations)\n");
    }
    extra_folders.push_str("  data/              ← shared conversations index\n");

    let content = format!(
r#"# ChatGPT History

> **For Claude — read this entire file before your first response.**

---

## What's here

My complete ChatGPT conversation history: **{count} conversations, {year_range}.**

```
ChatGPT History/
  2023/ 2024/ 2025/   ← conversations by year, one .md file each
{extra_folders}```

Each `.md` file is one conversation:

```
# Conversation Title
_YYYY-MM-DD_
---
[full conversation text]
```

---

## Your mission: help me build a working knowledge base

This is not just a history dump — I want to turn this into an active, organized workspace
so I can find past work, pick up where I left off, and hit the ground running.

Walk me through this in order. Don't skip ahead without my confirmation.

---

### Phase 1 — Scan and understand

Do this immediately when I connect you:

1. List all year folders and count the `.md` files in each
2. Skim 20–30 file titles spread across different years
3. Look specifically for: recurring project names, unfinished threads, topics I kept returning to
4. Report back with:
   - Total conversations and years covered
   - 5–8 major themes or projects you identified from the titles
   - 2–3 conversations that look like they might be unfinished or worth picking up

---

### Phase 2 — Propose a project structure

Based on what you found, propose a folder structure to replace the year folders.
Present it clearly so I can react to it. Example format:

```
Projects/
  [Project Name]/     ← named after a recurring project or product
  [Project Name]/
Topics/
  [Theme]/            ← recurring subject area (not a specific project)
  [Theme]/
Archive/              ← older one-off conversations not tied to current work
```

Ask me: "Does this structure look right? Anything to rename, merge, or add?"
Wait for my approval before doing anything.

---

### Phase 3 — Organize the files

Once I approve the structure:

1. Create the folders using your filesystem tools
2. Move conversations into the right folders — use titles and dates to decide
3. Tell me what you moved and flag anything you weren't sure about
4. **Do not delete anything.** If a file doesn't fit, put it in Archive/

---

### Phase 4 — Surface what to pick up

After organizing, give me:

- The 3–5 conversations that look most like active, unfinished work
- A one-line summary of each and where to find it
- Your recommendation for what to tackle first

---

## About Me

> Fill this in before starting — it helps Claude make better decisions about what matters

{name_line}
- **Role:**
- **Current focus:**
- **Tools / stack I use:**

### Topics I work on frequently

{topics_list}

### Context for Claude

<!-- Ongoing projects, priorities, what "active work" means for me -->

---

## Starter prompts once you're set up

```
What was I in the middle of when I stopped using ChatGPT?
Find everything related to [project or topic]
What should I pick up first?
Summarize what I was building in [year]
What questions do I ask Claude most often?
What have I been trying to figure out for a long time?
```

---

_Generated from {count} conversations ({year_range}) · edit freely, this file won't be overwritten_
"#,
        count = conversations.len(),
        year_range = year_range,
        extra_folders = extra_folders,
        topics_list = topics_list,
        name_line = name_line,
    );

    std::fs::write(root.join("README.md"), content).map_err(|e| e.to_string())
}

// ── Topic extraction ──────────────────────────────────────────────────────────

fn extract_top_topics(titles: &[String], top_n: usize) -> Vec<String> {
    const STOPWORDS: &[&str] = &[
        "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for",
        "of", "with", "about", "is", "are", "was", "were", "be", "been",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "can", "this", "that", "these", "those",
        "it", "its", "how", "what", "when", "where", "why", "which", "who",
        "my", "your", "our", "their", "his", "her", "from", "by", "as",
        "into", "more", "some", "any", "all", "new", "use", "using", "get",
        "help", "create", "make", "need", "vs", "based", "idea", "update",
        "analysis", "overview", "review", "discussion", "request", "draft",
        "via", "per", "re", "also", "just", "not", "no", "yes", "i",
        "add", "fix", "build", "work", "test", "run", "set", "check",
        "want", "take", "see", "look", "show", "try", "keep", "change",
    ];

    let mut freq: HashMap<String, usize> = HashMap::new();
    let mut orig_votes: HashMap<String, HashMap<String, usize>> = HashMap::new();

    for title in titles {
        let mut seen_in_title = std::collections::HashSet::new();
        for raw in title.split_whitespace() {
            let clean_orig: String = raw.chars().filter(|c| c.is_alphabetic()).collect();
            let clean_lower = clean_orig.to_lowercase();
            if clean_lower.len() > 3
                && !STOPWORDS.contains(&clean_lower.as_str())
                && seen_in_title.insert(clean_lower.clone())
            {
                *freq.entry(clean_lower.clone()).or_insert(0) += 1;
                *orig_votes
                    .entry(clean_lower)
                    .or_default()
                    .entry(clean_orig)
                    .or_insert(0) += 1;
            }
        }
    }

    let mut sorted: Vec<(String, usize)> = freq.into_iter()
        .filter(|(_, count)| *count >= 3)
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    sorted.iter().take(top_n).filter_map(|(lower, _)| {
        orig_votes.get(lower).map(|votes| {
            let mut v: Vec<(&String, &usize)> = votes.iter().collect();
            v.sort_by(|a, b| b.1.cmp(a.1));
            v[0].0.clone()
        })
    }).collect()
}

// ── Claude Desktop MCP configuration ──────────────────────────────────────────

fn update_claude_desktop_config(home: &str, export_path: &str) -> Result<(), String> {
    let config_path = PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Claude")
        .join("claude_desktop_config.json");

    let claude_dir = config_path.parent().unwrap();
    if !claude_dir.exists() {
        return Err("Claude Desktop not found".to_string());
    }

    let mut config: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if config.get("mcpServers").is_none() {
        config["mcpServers"] = serde_json::json!({});
    }

    config["mcpServers"]["chatgpt-history"] = serde_json::json!({
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-filesystem", export_path]
    });

    let json_str = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, json_str).map_err(|e| e.to_string())?;

    Ok(())
}

// ── Date helpers ──────────────────────────────────────────────────────────────

fn unix_to_year(ts: i64) -> String {
    let mut days = ts / 86_400;
    let mut year = 1970i32;
    loop {
        let days_in_year = if is_leap(year) { 366i64 } else { 365i64 };
        if days < days_in_year { break; }
        days -= days_in_year;
        year += 1;
    }
    year.to_string()
}

fn unix_to_date_str(ts: i64) -> String {
    let mut days = ts / 86_400;
    let mut year = 1970i32;
    loop {
        let days_in_year = if is_leap(year) { 366i64 } else { 365i64 };
        if days < days_in_year { break; }
        days -= days_in_year;
        year += 1;
    }
    let month_lengths: [i64; 12] = [
        31, if is_leap(year) { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut month = 1u32;
    for &m in &month_lengths {
        if days < m { break; }
        days -= m;
        month += 1;
    }
    format!("{}-{:02}-{:02}", year, month, days + 1)
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

// ── File naming ───────────────────────────────────────────────────────────────

fn slugify(s: &str) -> String {
    let slug: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_lowercase().next().unwrap_or('-') } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|seg| !seg.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    slug.chars().take(60).collect()
}
