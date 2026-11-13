//! Turning plain text edits back into values.

mod common;

use common::change;

#[test]
fn typing_plain_text() {
    let mut value = String::new();
    value = change(&value, "Hello ");
    assert_eq!(value, "Hello ");
    value = change(&value, "Hello world");
    assert_eq!(value, "Hello world");
}

#[test]
fn backspace_outside_and_inside_a_mention() {
    assert_eq!(
        change("Hello <@123> world", "Hello @123 worl"),
        "Hello <@123> worl"
    );
    assert_eq!(change("Hello <@123> world", "Hello @12"), "Hello @12");
}

#[test]
fn keeps_mentions_when_appending() {
    assert_eq!(change("Hi <@123>", "Hi @123!"), "Hi <@123>!");
}

#[test]
fn keeps_mentions_when_prepending() {
    assert_eq!(change("Hi <@123>", "Hey Hi @123"), "Hey Hi <@123>");
}

#[test]
fn keeps_mentions_when_inserting_between_parts() {
    assert_eq!(
        change("Hello <@123> world", "Hello @123 beautiful world"),
        "Hello <@123> beautiful world"
    );
}

#[test]
fn keeps_mentions_when_deleting_outside_them() {
    assert_eq!(
        change("Hello <@123> world", "Helo @123 world"),
        "Helo <@123> world"
    );
}

#[test]
fn keeps_a_trailing_mention_when_the_text_before_it_is_deleted() {
    assert_eq!(change("some text <@123>", "@123"), "<@123>");
}

#[test]
fn keeps_a_leading_mention_when_the_text_after_it_is_deleted() {
    assert_eq!(change("<@123> some text", "@123"), "<@123>");
}

#[test]
fn keeps_the_mentions_around_a_replaced_word() {
    assert_eq!(change("<@1> old <@2>", "@1 new @2"), "<@1> new <@2>");
}

#[test]
fn keeps_the_mentions_around_an_inserted_word() {
    assert_eq!(
        change("<@1> and <@2>", "@1 and also @2"),
        "<@1> and also <@2>"
    );
}

#[test]
fn keeps_a_mention_inside_a_replaced_range() {
    assert_eq!(change("a <@1> b", "x @1 y"), "x <@1> y");
}

#[test]
fn an_edited_label_becomes_plain_text() {
    assert_eq!(
        change("Hello <@123> world", "Hello X world"),
        "Hello X world"
    );
    assert_eq!(
        change("Hello <@123> world", "Hello @1x23 world"),
        "Hello @1x23 world"
    );
}

#[test]
fn clearing_everything() {
    assert_eq!(change("Hello <@123>", ""), "");
}

#[test]
fn no_op_change() {
    assert_eq!(change("Hello <@123>", "Hello @123"), "Hello <@123>");
}

#[test]
fn middle_replacement_without_mentions() {
    assert_eq!(change("abcdef", "abXYef"), "abXYef");
}

#[test]
fn keeps_multi_code_point_emoji_intact() {
    let family = "👨‍👩‍👧‍👦";
    let wave = "👋🏻";

    assert_eq!(
        change(&format!("Hi {family} "), &format!("Hi {family} there")),
        format!("Hi {family} there")
    );
    assert_eq!(change(wave, &format!("{wave} hi")), format!("{wave} hi"));
    assert_eq!(change("👋 <@123>", "👋👋 @123"), "👋👋 <@123>");
    assert_eq!(change("👋 <@123>", "👋 @123!"), "👋 <@123>!");
    assert_eq!(change("é <@1> ü", "ü @1 é"), "ü <@1> é");
}
