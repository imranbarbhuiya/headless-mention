use std::ops::Range;

use crate::PartType;

/// A mention matched in a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MentionData {
    /// The `id` group, so the id of the suggestion that was picked.
    pub id: String,
    /// The `trigger` group, such as `@`.
    pub trigger: String,
    /// The text the pattern matched, such as `<@1>`, as opposed to the label shown for it.
    pub original: String,
}

/// A piece of a parsed value: a mention, a styled match or a run of plain text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    /// The text shown, which is the label for a mention.
    pub text: String,
    /// Where the part sits in the plain text, in bytes.
    pub range: Range<usize>,
    /// Where the part sits in the value, in bytes.
    pub value_range: Range<usize>,
    /// The index of the part type that matched, unset for plain text.
    pub part_type: Option<usize>,
    /// The mention this part holds, only set for parts matched by a mention type.
    pub mention: Option<MentionData>,
}

impl Part {
    /// The text this part stands for in the value: the original for a mention, the text otherwise.
    pub fn raw(&self) -> &str {
        self.mention
            .as_ref()
            .map_or(&self.text, |mention| &mention.original)
    }
}

/// A value split into parts, along with its plain text.
///
/// The plain text is what the user sees and edits: the value with every mention replaced by its
/// label. The parts cover it from start to end without gaps, and an empty value has one empty part.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parsed {
    /// The value as shown, with every mention replaced by its label.
    pub plain_text: String,
    /// The parts, in order.
    pub parts: Vec<Part>,
}

impl Parsed {
    /// The mentions in the value, in order.
    pub fn mentions(&self) -> impl Iterator<Item = &MentionData> {
        self.parts.iter().filter_map(|part| part.mention.as_ref())
    }

    /// The value these parts stand for.
    pub fn value(&self) -> String {
        self.parts.iter().map(Part::raw).collect()
    }

    fn push(
        &mut self,
        text: String,
        value_range: Range<usize>,
        part_type: Option<usize>,
        mention: Option<MentionData>,
    ) {
        let start = self.plain_text.len();
        self.plain_text.push_str(&text);

        self.parts.push(Part {
            range: start..self.plain_text.len(),
            text,
            value_range,
            part_type,
            mention,
        });
    }
}

/// Splits `value` into parts using `part_types`, in order.
///
/// ```
/// use headless_mention::{MentionType, PartType, parse};
///
/// let types = [PartType::from(MentionType::new('@', |mention| format!("@user{}", mention.id)))];
/// let parsed = parse("hi <@1>!", &types);
///
/// assert_eq!(parsed.plain_text, "hi @user1!");
/// assert_eq!(parsed.mentions().next().unwrap().id, "1");
/// ```
pub fn parse(value: &str, part_types: &[PartType]) -> Parsed {
    let mut parsed = Parsed::default();
    parse_into(value, part_types, 0, 0, &mut parsed);

    if parsed.parts.is_empty() {
        parsed.push(String::new(), 0..0, None, None);
    }

    parsed
}

fn parse_into(
    value: &str,
    part_types: &[PartType],
    index: usize,
    value_offset: usize,
    parsed: &mut Parsed,
) {
    let Some(part_type) = part_types.get(index) else {
        if !value.is_empty() {
            parsed.push(
                value.to_owned(),
                value_offset..value_offset + value.len(),
                None,
                None,
            );
        }
        return;
    };

    let mut last = 0;

    part_type.pattern().for_each_match(value, &mut |found| {
        // An empty match has nothing to show, and a match overlapping the last one is a broken matcher.
        if found.range.is_empty() || found.range.start < last || found.range.end > value.len() {
            return;
        }

        if found.range.start > last {
            parse_into(
                &value[last..found.range.start],
                part_types,
                index + 1,
                value_offset + last,
                parsed,
            );
        }

        let text = &value[found.range.clone()];
        let value_range = value_offset + found.range.start..value_offset + found.range.end;

        match part_type {
            PartType::Mention(mention_type) => {
                let group = |range: Option<Range<usize>>| {
                    range.and_then(|range| value.get(range)).unwrap_or_default()
                };
                let trigger = group(found.trigger);

                if trigger == mention_type.trigger {
                    let mention = MentionData {
                        id: group(found.id).to_owned(),
                        trigger: trigger.to_owned(),
                        original: text.to_owned(),
                    };

                    parsed.push(
                        mention_type.label(&mention),
                        value_range,
                        Some(index),
                        Some(mention),
                    );
                } else {
                    parse_into(text, part_types, index + 1, value_range.start, parsed);
                }
            }
            PartType::Pattern(_) => parsed.push(text.to_owned(), value_range, Some(index), None),
        }

        last = found.range.end;
    });

    if last < value.len() {
        parse_into(
            &value[last..],
            part_types,
            index + 1,
            value_offset + last,
            parsed,
        );
    }
}
