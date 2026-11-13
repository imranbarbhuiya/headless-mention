//! Part types and helpers shared by the tests.
#![allow(dead_code)]

use headless_mention::{MentionType, PartType, PatternType, apply_text_change, parse};
use regex::Regex;

pub fn users() -> MentionType {
    MentionType::new('@', |mention| format!("@{}", mention.id))
}

pub fn bold() -> PatternType {
    PatternType::new(Regex::new(r"\*\*\S(?:.*?\S)?\*\*").unwrap())
}

pub fn types() -> Vec<PartType> {
    vec![users().into()]
}

pub fn change(value: &str, text: &str) -> String {
    apply_text_change(&parse(value, &types()), text)
}
