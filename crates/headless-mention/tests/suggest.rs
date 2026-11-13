//! Suggestion keywords and picking a suggestion.

mod common;

use std::ops::Range;

use common::users;
use headless_mention::{Insertion, MentionType, PatternType, insert_mention, keyword, parse};
use regex::Regex;

fn keyword_at(
    value: &str,
    mention_type: &MentionType,
    selection: Option<Range<usize>>,
) -> Option<String> {
    let parsed = parse(value, &[mention_type.clone().into()]);
    let end = parsed.plain_text.len();

    keyword(&parsed, mention_type, selection.unwrap_or(end..end)).map(str::to_owned)
}

fn insert_at_end(value: &str, mention_type: &MentionType, id: &str) -> Option<Insertion> {
    let parsed = parse(value, &[mention_type.clone().into()]);
    let end = parsed.plain_text.len();

    insert_mention(&parsed, mention_type, end..end, id)
}

#[test]
fn keyword_after_the_trigger() {
    assert_eq!(
        keyword_at("Hello @jo", &users(), None).as_deref(),
        Some("jo")
    );
}

#[test]
fn empty_keyword_right_after_the_trigger() {
    assert_eq!(keyword_at("Hello @", &users(), None).as_deref(), Some(""));
    assert_eq!(keyword_at("@", &users(), None).as_deref(), Some(""));
}

#[test]
fn spaces_within_the_allowed_count() {
    assert_eq!(
        keyword_at("Hello @john doe", &users(), None).as_deref(),
        Some("john doe")
    );
}

#[test]
fn no_keyword_for_a_range_selection() {
    assert_eq!(keyword_at("Hello @jo", &users(), Some(6..9)), None);
}

#[test]
fn no_keyword_without_whitespace_before_the_trigger() {
    assert_eq!(keyword_at("Hello@jo", &users(), None), None);
}

#[test]
fn no_keyword_past_the_allowed_spaces() {
    assert_eq!(keyword_at("Hello @john doe smith", &users(), None), None);
    assert_eq!(
        keyword_at("Hello @john doe smith", &users().allowed_spaces(2), None).as_deref(),
        Some("john doe smith")
    );
    assert_eq!(
        keyword_at("Hello @john doe", &users().allowed_spaces(0), None),
        None
    );
}

#[test]
fn a_new_line_ends_the_keyword() {
    assert_eq!(keyword_at("Hello @jo\nhn", &users(), None), None);
    assert_eq!(
        keyword_at("Hello\n@jo", &users(), None).as_deref(),
        Some("jo")
    );
}

#[test]
fn no_keyword_inside_or_right_after_a_mention() {
    assert_eq!(keyword_at("Hello <@123>", &users(), None), None);
    assert_eq!(keyword_at("Hello <@123>", &users(), Some(8..8)), None);
}

#[test]
fn no_keyword_when_the_caret_is_before_the_trigger() {
    assert_eq!(keyword_at("a @b", &users(), Some(2..2)), None);
}

#[test]
fn keyword_inside_a_styled_part() {
    let bold = PatternType::new(Regex::new(r"\*\*.+?\*\*").unwrap());
    let parsed = parse("**hi @jo**", &[users().into(), bold.into()]);

    assert_eq!(keyword(&parsed, &users(), 8..8), Some("jo"));
}

#[test]
fn multi_char_trigger() {
    let tags = MentionType::new("::", |mention| format!("::{}", mention.id));

    assert_eq!(keyword_at("hi ::sm", &tags, None).as_deref(), Some("sm"));
    assert_eq!(
        insert_at_end("hi ::sm", &tags, "smile").unwrap().value,
        "hi <::smile>"
    );
}

#[test]
fn inserts_in_place_of_the_keyword() {
    let insertion = insert_at_end("Hello @jo", &users(), "123").unwrap();

    assert_eq!(insertion.value, "Hello <@123>");
    assert_eq!(insertion.value_caret, insertion.value.len());
}

#[test]
fn keeps_the_text_after_the_caret() {
    let parsed = parse("Hello @jo world", &[users().into()]);
    let insertion = insert_mention(&parsed, &users(), 9..9, "1").unwrap();

    assert_eq!(insertion.value, "Hello <@1> world");
    assert_eq!(insertion.value_caret, 10);
}

#[test]
fn keeps_mentions_before_and_after() {
    let parsed = parse("<@1> @ja <@2>", &[users().into()]);
    let insertion = insert_mention(&parsed, &users(), 6..6, "3").unwrap();

    assert_eq!(insertion.value, "<@1> <@3> <@2>");
}

#[test]
fn adds_a_space_at_the_end_when_enabled() {
    let users = users().space_after_mention(true);

    let at_end = insert_at_end("Hello @jo", &users, "1").unwrap();
    assert_eq!(at_end.value, "Hello <@1> ");
    assert_eq!(at_end.value_caret, at_end.value.len());

    let parsed = parse("@jo\nnext", &[users.clone().into()]);
    assert_eq!(
        insert_mention(&parsed, &users, 3..3, "1").unwrap().value,
        "<@1> \nnext"
    );

    let parsed = parse("@jo next", &[users.clone().into()]);
    assert_eq!(
        insert_mention(&parsed, &users, 3..3, "1").unwrap().value,
        "<@1> next"
    );
}

#[test]
fn custom_format() {
    let channels = MentionType::new('#', |mention| format!("#{}", mention.id)).pattern(
        Regex::new(r"\[(?P<trigger>#)(?P<id>\w+)\]").unwrap(),
        |trigger, id| format!("[{trigger}{id}]"),
    );

    let insertion = insert_at_end("join #gen", &channels, "general").unwrap();

    assert_eq!(insertion.value, "join [#general]");
    assert_eq!(
        parse(&insertion.value, &[channels.into()]).plain_text,
        "join #general"
    );
}

#[test]
fn nothing_to_insert_without_a_keyword() {
    assert_eq!(insert_at_end("Hello", &users(), "1"), None);

    let parsed = parse("Hello", &[users().into()]);
    assert_eq!(insert_mention(&parsed, &users(), 100..100, "1"), None);
}
