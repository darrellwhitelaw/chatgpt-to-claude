pub const PASS1_SYSTEM_PROMPT: &str = "You are a conversation analyst. \
Given a list of conversation titles and brief excerpts from a ChatGPT export, \
identify 5-20 distinct topical cluster labels that would meaningfully organize \
this conversation history. Each label should be 2-4 words, clear, and non-overlapping. \
Return ONLY a JSON object with one field: {\"labels\": [\"Label 1\", \"Label 2\", ...]}. \
No other text.";

pub fn build_pass1_message(titles_and_snippets: &str) -> String {
    format!(
        "Here are conversation titles and opening lines from a ChatGPT export:\n\n{}\n\n\
         Generate 5-20 cluster labels for organizing these conversations.",
        titles_and_snippets
    )
}

/// pass2_system is built dynamically with the cluster vocabulary embedded
pub fn build_pass2_system(cluster_labels: &[String]) -> String {
    let labels_list = cluster_labels
        .iter()
        .enumerate()
        .map(|(i, l)| format!("{}. {}", i + 1, l))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "You are analyzing a ChatGPT conversation transcript. \
Return ONLY a JSON object with exactly these three fields:\n\
- \"cluster_label\": string — choose from ONLY these options:\n{}\n\
- \"summary\": string — 3-5 sentences covering the main topic, key decisions, and conclusions. \
Plain English, no jargon.\n\
- \"instructions\": string or null — any custom instructions the user gave the AI (e.g. \
'always respond in bullet points', 'use metric units', preferred tone/format). \
Extract from system prompts or explicit user instructions. null if none found.\n\
No other text, no markdown, just valid JSON.",
        labels_list
    )
}

pub fn build_pass2_user_message(full_text: &str) -> String {
    // Truncate to prevent batch size issues (Pitfall 4)
    const MAX_CHARS: usize = 8_000;
    let text = if full_text.len() > MAX_CHARS {
        &full_text[..MAX_CHARS]
    } else {
        full_text
    };
    format!("Conversation transcript:\n\n{}", text)
}
