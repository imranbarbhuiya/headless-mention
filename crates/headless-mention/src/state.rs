use std::ops::Range;

use crate::{
    MentionData, MentionType, Parsed, Part, PartType, apply_text_change, insert_mention, keyword,
    parse,
};

/// The state of one mention input: its value, its parsed parts and its selection.
///
/// Feed it what the text field reports and render what it holds:
///
/// 1. Show [`MentionState::plain_text`] in the field, styling each of [`MentionState::parts`] by its
///    `part_type`.
/// 2. On every edit, call [`MentionState::edit`] with the field's new text and selection. When it
///    returns `true`, write [`MentionState::plain_text`] and [`MentionState::selection`] back
///    to the field.
/// 3. Show suggestions while [`MentionState::keyword`] returns one, and call
///    [`MentionState::insert_mention`] with the picked id, then write the text and selection back.
/// 4. Store or send [`MentionState::value`].
///
/// ```
/// use headless_mention::{MentionState, MentionType};
///
/// let mut state = MentionState::new([MentionType::new('@', |mention| format!("@user{}", mention.id)).into()]);
///
/// state.edit("hi @jo", 6..6);
/// assert_eq!(state.keyword("@"), Some("jo"));
///
/// assert!(state.insert_mention("@", "42"));
/// assert_eq!(state.value(), "hi <@42>");
/// assert_eq!(state.plain_text(), "hi @user42");
/// assert_eq!(state.selection(), 10..10);
/// ```
#[derive(Clone, Debug)]
pub struct MentionState {
    part_types: Vec<PartType>,
    value: String,
    parsed: Parsed,
    selection: Range<usize>,
}

impl MentionState {
    /// An empty input looking for `part_types`, in order.
    pub fn new(part_types: impl Into<Vec<PartType>>) -> Self {
        let part_types = part_types.into();

        Self {
            parsed: parse("", &part_types),
            part_types,
            value: String::new(),
            selection: 0..0,
        }
    }

    /// Sets the starting value.
    #[must_use]
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.set_value(value);
        self
    }

    /// The value, with mentions written as `<@id>`.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// The text as shown, with every mention replaced by its label.
    pub fn plain_text(&self) -> &str {
        &self.parsed.plain_text
    }

    /// The parts of the value, in order.
    pub fn parts(&self) -> &[Part] {
        &self.parsed.parts
    }

    /// The parsed value.
    pub fn parsed(&self) -> &Parsed {
        &self.parsed
    }

    /// The part types, in the order they're applied.
    pub fn part_types(&self) -> &[PartType] {
        &self.part_types
    }

    /// The mentions in the value, in order.
    pub fn mentions(&self) -> impl Iterator<Item = &MentionData> {
        self.parsed.mentions()
    }

    /// The selection in the plain text, in bytes.
    pub fn selection(&self) -> Range<usize> {
        self.selection.clone()
    }

    /// Replaces the value, such as clearing it after sending, and keeps the selection in bounds.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.parsed = parse(&self.value, &self.part_types);
        self.set_selection(self.selection.clone());
    }

    /// Moves the selection, given in the plain text.
    ///
    /// It's kept in bounds and on char boundaries.
    pub fn set_selection(&mut self, selection: Range<usize>) {
        let text = &self.parsed.plain_text;
        let end = floor_char_boundary(text, selection.end);
        let start = floor_char_boundary(text, selection.start).min(end);

        self.selection = start..end;
    }

    /// Applies an edit the user made: the field now shows `text` with `selection`.
    ///
    /// Returns `true` when the plain text or selection now differs from what the field shows, for
    /// example because typing `<@1>` made a mention. Write both back to the field when it does.
    pub fn edit(&mut self, text: &str, selection: Range<usize>) -> bool {
        if text != self.parsed.plain_text {
            let value = apply_text_change(&self.parsed, text);
            self.set_value(value);
        }

        self.set_selection(selection.clone());

        self.parsed.plain_text != text || self.selection != selection
    }

    /// The keyword typed after `trigger` at the caret, if suggestions for it should show.
    ///
    /// See [`keyword`](crate::keyword).
    pub fn keyword(&self, trigger: &str) -> Option<&str> {
        keyword(
            &self.parsed,
            self.mention_type(trigger)?,
            self.selection.clone(),
        )
    }

    /// The trigger and keyword of whichever mention type is being typed, if any.
    pub fn active_keyword(&self) -> Option<(&str, &str)> {
        self.part_types
            .iter()
            .filter_map(PartType::as_mention)
            .find_map(|mention_type| {
                keyword(&self.parsed, mention_type, self.selection.clone())
                    .map(|keyword| (mention_type.trigger(), keyword))
            })
    }

    /// Replaces the keyword being typed after `trigger` with a mention of `id`, and moves the caret
    /// past it.
    ///
    /// Returns `false` and changes nothing when there's no keyword for `trigger`.
    pub fn insert_mention(&mut self, trigger: &str, id: &str) -> bool {
        let Some(mention_type) = self.mention_type(trigger) else {
            return false;
        };
        let Some(insertion) =
            insert_mention(&self.parsed, mention_type, self.selection.clone(), id)
        else {
            return false;
        };

        self.set_value(insertion.value);
        let caret = self
            .parsed
            .to_plain_offset(insertion.value_caret, crate::Bias::End);
        self.selection = caret..caret;

        true
    }

    fn mention_type(&self, trigger: &str) -> Option<&MentionType> {
        self.part_types
            .iter()
            .filter_map(PartType::as_mention)
            .find(|mention_type| mention_type.trigger() == trigger)
    }
}

fn floor_char_boundary(text: &str, offset: usize) -> usize {
    if offset >= text.len() {
        return text.len();
    }

    (0..=offset)
        .rev()
        .find(|&offset| text.is_char_boundary(offset))
        .unwrap_or(0)
}
