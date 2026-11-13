//! Mapping offsets and selections between the plain text and the value.

mod common;

use common::types;
use headless_mention::{Bias, parse};

// Plain text: "hi @123 yo", value: "hi <@123> yo".
fn parsed() -> headless_mention::Parsed {
    parse("hi <@123> yo", &types())
}

#[test]
fn plain_text_offsets_map_one_to_one() {
    let parsed = parsed();

    assert_eq!(parsed.to_value_offset(0, Bias::Start), 0);
    assert_eq!(parsed.to_value_offset(2, Bias::Start), 2);
    assert_eq!(parsed.to_value_offset(9, Bias::Start), 11);
    assert_eq!(parsed.to_value_offset(10, Bias::Start), 12);
}

#[test]
fn mention_edges_map_to_mention_edges() {
    let parsed = parsed();

    assert_eq!(parsed.to_value_offset(3, Bias::End), 3);
    assert_eq!(parsed.to_value_offset(7, Bias::Start), 9);
    assert_eq!(parsed.to_plain_offset(3, Bias::End), 3);
    assert_eq!(parsed.to_plain_offset(9, Bias::Start), 7);
}

#[test]
fn offsets_inside_a_mention_snap_by_bias() {
    let parsed = parsed();

    assert_eq!(parsed.to_value_offset(5, Bias::Start), 3);
    assert_eq!(parsed.to_value_offset(5, Bias::End), 9);
    assert_eq!(parsed.to_plain_offset(6, Bias::Start), 3);
    assert_eq!(parsed.to_plain_offset(6, Bias::End), 7);
}

#[test]
fn offsets_past_the_end_clamp() {
    let parsed = parsed();

    assert_eq!(parsed.to_value_offset(100, Bias::Start), 12);
    assert_eq!(parsed.to_plain_offset(100, Bias::Start), 10);
}

#[test]
fn a_selection_partly_over_a_mention_grows() {
    let parsed = parsed();

    assert_eq!(parsed.to_value_range(0..5), 0..9);
    assert_eq!(parsed.to_value_range(5..10), 3..12);
    assert_eq!(parsed.to_plain_range(4..7), 3..7);
}

#[test]
fn a_caret_inside_a_mention_moves_past_it() {
    let parsed = parsed();

    assert_eq!(parsed.to_value_range(5..5), 9..9);
    assert_eq!(parsed.to_plain_range(6..6), 7..7);
}

#[test]
fn wrapping_a_selection_in_bold() {
    let parsed = parsed();
    let value = parsed.value();

    // The user selects "@123 y" in the plain text.
    let selection = parsed.to_value_range(3..9);
    let wrapped = format!(
        "{}**{}**{}",
        &value[..selection.start],
        &value[selection.clone()],
        &value[selection.end..]
    );

    assert_eq!(wrapped, "hi **<@123> y**o");
}
