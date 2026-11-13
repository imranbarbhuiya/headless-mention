//! The `MentionState` wrapper driving one input.

mod common;

use common::{bold, types, users};
use headless_mention::{MentionState, MentionType};

#[test]
fn starts_empty_with_the_caret_at_the_start() {
    let state = MentionState::new(types());

    assert_eq!(state.value(), "");
    assert_eq!(state.plain_text(), "");
    assert_eq!(state.parts().len(), 1);
    assert_eq!(state.selection(), 0..0);
}

#[test]
fn parses_the_starting_value() {
    let state = MentionState::new(types()).with_value("hi <@1>");

    assert_eq!(state.plain_text(), "hi @1");
    assert_eq!(
        state
            .mentions()
            .map(|mention| mention.id.as_str())
            .collect::<Vec<_>>(),
        ["1"]
    );
}

#[test]
fn an_edit_keeps_untouched_mentions() {
    let mut state = MentionState::new(types()).with_value("hi <@1>");

    assert!(!state.edit("hi @1!", 6..6));
    assert_eq!(state.value(), "hi <@1>!");
    assert_eq!(state.selection(), 6..6);
}

#[test]
fn an_edit_inside_a_label_drops_the_mention() {
    let mut state = MentionState::new(types()).with_value("hi <@12>");

    assert!(!state.edit("hi @1", 5..5));
    assert_eq!(state.value(), "hi @1");
    assert_eq!(state.mentions().count(), 0);
}

#[test]
fn an_edit_that_makes_a_mention_asks_for_a_write_back() {
    let mut state = MentionState::new(types());

    assert!(state.edit("hi <@1>", 7..7));
    assert_eq!(state.plain_text(), "hi @1");
    assert_eq!(state.selection(), 5..5);
}

#[test]
fn selection_is_kept_in_bounds_and_on_char_boundaries() {
    let mut state = MentionState::new(types()).with_value("é");

    state.set_selection(1..1);
    assert_eq!(state.selection(), 0..0);

    state.set_selection(5..9);
    assert_eq!(state.selection(), 2..2);

    // A reversed selection, as some fields report while dragging backwards.
    state.set_selection(std::ops::Range { start: 2, end: 0 });
    assert_eq!(state.selection(), 0..0);
}

#[test]
fn clearing_the_value_moves_the_caret_back() {
    let mut state = MentionState::new(types()).with_value("hello");
    state.set_selection(5..5);

    state.set_value("");

    assert_eq!(state.plain_text(), "");
    assert_eq!(state.selection(), 0..0);
}

#[test]
fn suggestion_flow() {
    let mut state = MentionState::new(types());

    state.edit("Hello @", 7..7);
    assert_eq!(state.keyword("@"), Some(""));

    state.edit("Hello @jo", 9..9);
    assert_eq!(state.keyword("@"), Some("jo"));
    assert_eq!(state.active_keyword(), Some(("@", "jo")));

    assert!(state.insert_mention("@", "123"));
    assert_eq!(state.value(), "Hello <@123>");
    assert_eq!(state.plain_text(), "Hello @123");
    assert_eq!(state.selection(), 10..10);
    assert_eq!(state.keyword("@"), None);
}

#[test]
fn several_mentions() {
    let mut state = MentionState::new([users().space_after_mention(true).into(), bold().into()]);

    state.edit("@ad", 3..3);
    assert!(state.insert_mention("@", "1"));
    assert_eq!(state.value(), "<@1> ");
    assert_eq!(state.selection(), 3..3);

    state.edit("@1 **and** @gr", 14..14);
    assert!(state.insert_mention("@", "2"));
    assert_eq!(state.value(), "<@1> **and** <@2> ");
    assert_eq!(state.plain_text(), "@1 **and** @2 ");
    assert_eq!(state.selection(), 14..14);
}

#[test]
fn picks_the_mention_type_by_trigger() {
    let channels = MentionType::new('#', |mention| format!("#{}", mention.id));
    let mut state = MentionState::new([users().into(), channels.into()]);

    state.edit("go #ge", 6..6);
    assert_eq!(state.keyword("@"), None);
    assert_eq!(state.active_keyword(), Some(("#", "ge")));

    assert!(!state.insert_mention("@", "1"));
    assert!(!state.insert_mention("!", "1"));
    assert!(state.insert_mention("#", "general"));
    assert_eq!(state.value(), "go <#general>");
}
