use crate::pipeline::json_parser::{Message, MessageNode};
use std::collections::HashMap;

pub fn linearize_messages(
    _mapping: &HashMap<String, MessageNode>,
    _current_node: &str,
) -> Vec<Message> {
    vec![] // Stub — tests will fail (RED)
}

pub fn should_include_message(_msg: &Message) -> bool {
    false // Stub
}
