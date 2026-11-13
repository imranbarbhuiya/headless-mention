//! Parsing values into parts.

mod common;

use common::{bold, types, users};
use headless_mention::{MentionData, MentionType, Part, PartType, PatternType, parse};
use regex::Regex;

#[test]
fn empty_value_has_one_empty_part() {
    let parsed = parse("", &[]);

    assert_eq!(parsed.plain_text, "");
    assert_eq!(
        parsed.parts,
        [Part {
            text: String::new(),
            range: 0..0,
            value_range: 0..0,
            part_type: None,
            mention: None,
        }]
    );
}

#[test]
fn plain_text_without_part_types() {
    let parsed = parse("Hello world", &[]);

    assert_eq!(parsed.plain_text, "Hello world");
    assert_eq!(parsed.parts.len(), 1);
    assert_eq!(parsed.parts[0].range, 0..11);
}

#[test]
fn mentions_become_labels() {
    let parsed = parse("Hello <@123> world", &types());

    assert_eq!(parsed.plain_text, "Hello @123 world");
    assert_eq!(parsed.parts.len(), 3);

    let mention = &parsed.parts[1];
    assert_eq!(mention.text, "@123");
    assert_eq!(mention.range, 6..10);
    assert_eq!(mention.value_range, 6..12);
    assert_eq!(mention.part_type, Some(0));
    assert_eq!(
        mention.mention,
        Some(MentionData {
            id: "123".into(),
            trigger: "@".into(),
            original: "<@123>".into(),
        })
    );
}

#[test]
fn non_matching_patterns_leave_plain_text() {
    let parsed = parse("Hello world", &types());

    assert_eq!(parsed.plain_text, "Hello world");
    assert_eq!(parsed.parts.len(), 1);
    assert_eq!(parsed.parts[0].part_type, None);
}

#[test]
fn mixed_mentions_and_patterns() {
    let parsed = parse("**hi** <@1> and <@2>!", &[users().into(), bold().into()]);

    assert_eq!(parsed.plain_text, "**hi** @1 and @2!");
    let texts: Vec<_> = parsed
        .parts
        .iter()
        .map(|part| (part.text.as_str(), part.part_type))
        .collect();
    assert_eq!(
        texts,
        [
            ("**hi**", Some(1)),
            (" ", None),
            ("@1", Some(0)),
            (" and ", None),
            ("@2", Some(0)),
            ("!", None),
        ]
    );
}

#[test]
fn a_match_is_never_parsed_again() {
    // The bold pattern runs first and claims the mention inside it.
    let parsed = parse("**<@1>**", &[bold().into(), users().into()]);

    assert_eq!(parsed.plain_text, "**<@1>**");
    assert_eq!(parsed.mentions().count(), 0);
}

#[test]
fn another_trigger_is_left_to_later_types() {
    let any_trigger = MentionType::new('@', |mention| format!("@{}", mention.id)).pattern(
        Regex::new(r"<(?P<trigger>[@#])(?P<id>\w+)>").unwrap(),
        |trigger, id| format!("<{trigger}{id}>"),
    );
    let channels = MentionType::new('#', |mention| format!("#{}", mention.id));

    let parsed = parse("<@1> <#general>", &[any_trigger.into(), channels.into()]);

    assert_eq!(parsed.plain_text, "@1 #general");
    let triggers: Vec<_> = parsed
        .mentions()
        .map(|mention| mention.trigger.as_str())
        .collect();
    assert_eq!(triggers, ["@", "#"]);
    assert_eq!(parsed.parts[2].part_type, Some(1));
}

#[test]
fn unnamed_groups_are_read_in_order() {
    let mention = MentionType::new('@', |mention| format!("@{}", mention.id))
        .pattern(Regex::new(r"\{(@)(\d+)\}").unwrap(), |trigger, id| {
            format!("{{{trigger}{id}}}")
        });

    let parsed = parse("hi {@7}", &[mention.into()]);

    assert_eq!(parsed.plain_text, "hi @7");
    assert_eq!(parsed.mentions().next().unwrap().id, "7");
}

#[test]
fn value_round_trips() {
    let types: Vec<PartType> = vec![users().into(), bold().into()];
    let value = "👋 **hi** <@1>\n<@2> done";

    assert_eq!(parse(value, &types).value(), value);
}

#[test]
fn empty_matches_are_skipped() {
    let parsed = parse("abc", &[PatternType::new(Regex::new("x*").unwrap()).into()]);

    assert_eq!(parsed.plain_text, "abc");
    assert_eq!(parsed.parts.len(), 1);
}

#[cfg(feature = "fancy-regex")]
#[test]
fn fancy_regex_look_around() {
    let italic = PatternType::new(
        fancy_regex::Regex::new(r"(?<!\w)_(?<text>\S(?:.*?\S)??)_(?!\w)").unwrap(),
    );

    let parsed = parse("_yes_ but not snake_case_name", &[italic.into()]);

    let styled: Vec<_> = parsed
        .parts
        .iter()
        .filter(|part| part.part_type.is_some())
        .map(|part| part.text.as_str())
        .collect();
    assert_eq!(styled, ["_yes_"]);
}
