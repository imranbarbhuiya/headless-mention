use std::{fmt, sync::Arc};

use crate::{MentionData, Pattern};

type LabelFn = dyn Fn(&MentionData) -> String + Send + Sync;
type FormatFn = dyn Fn(&str, &str) -> String + Send + Sync;

/// A mention: an atomic token written into the value as `<@id>` and shown to the user as a label.
///
/// Editing the label breaks the mention apart into plain text, so the label should be short and
/// stable, such as `@Ada`.
///
/// ```
/// use headless_mention::MentionType;
///
/// let users = MentionType::new('@', |mention| format!("@user{}", mention.id))
///     .allowed_spaces(2)
///     .space_after_mention(true);
/// ```
#[derive(Clone)]
pub struct MentionType {
    pub(crate) trigger: String,
    pub(crate) pattern: Arc<dyn Pattern>,
    pub(crate) format: Arc<FormatFn>,
    pub(crate) label: Arc<LabelFn>,
    pub(crate) allowed_spaces: usize,
    pub(crate) space_after: bool,
}

impl MentionType {
    /// A mention type started by `trigger` and shown as whatever `label` returns.
    ///
    /// Mentions are written as `<{trigger}{id}>`, where the id is anything up to the closing `>`
    /// except whitespace and angle brackets. Use [`MentionType::pattern`] for another format.
    ///
    /// # Panics
    ///
    /// Panics if `trigger` is empty.
    pub fn new(
        trigger: impl Into<String>,
        label: impl Fn(&MentionData) -> String + Send + Sync + 'static,
    ) -> Self {
        let trigger = trigger.into();
        assert!(!trigger.is_empty(), "a mention trigger can't be empty");

        let pattern = regex::Regex::new(&format!(
            "<(?P<trigger>{})(?P<id>[^<>\\s]+)>",
            regex::escape(&trigger)
        ))
        .expect("an escaped trigger always makes a valid pattern");

        Self {
            trigger,
            pattern: Arc::new(pattern),
            format: Arc::new(|trigger, id| format!("<{trigger}{id}>")),
            label: Arc::new(label),
            allowed_spaces: 1,
            space_after: false,
        }
    }

    /// Replaces how mentions are matched and written.
    ///
    /// `pattern` needs a `trigger` and an `id` group (see [`Pattern`]), and `format` builds the text
    /// it matches from a trigger and an id. The two have to agree, or a picked suggestion won't
    /// parse back into a mention.
    ///
    /// ```
    /// use headless_mention::MentionType;
    ///
    /// let channels = MentionType::new('#', |mention| format!("#{}", mention.id)).pattern(
    ///     regex::Regex::new(r"\[(?P<trigger>#)(?P<id>\w+)\]").unwrap(),
    ///     |trigger, id| format!("[{trigger}{id}]"),
    /// );
    /// ```
    #[must_use]
    pub fn pattern(
        mut self,
        pattern: impl Pattern + 'static,
        format: impl Fn(&str, &str) -> String + Send + Sync + 'static,
    ) -> Self {
        self.pattern = Arc::new(pattern);
        self.format = Arc::new(format);
        self
    }

    /// How many spaces the keyword may contain before suggestions stop.
    ///
    /// They're counted between the trigger and the caret, so the default of `1` still suggests
    /// for `@john doe` and stops at `@john doe smith`. A new line always ends the keyword.
    #[must_use]
    pub fn allowed_spaces(mut self, count: usize) -> Self {
        self.allowed_spaces = count;
        self
    }

    /// Whether to add a space after a mention picked at the end of the text or of a line.
    ///
    /// Off by default.
    #[must_use]
    pub fn space_after_mention(mut self, enabled: bool) -> Self {
        self.space_after = enabled;
        self
    }

    /// The text that starts a mention, such as `@`.
    pub fn trigger(&self) -> &str {
        &self.trigger
    }

    /// The text written into the value for a mention of `id`, such as `<@42>`.
    pub fn format(&self, id: &str) -> String {
        (self.format)(&self.trigger, id)
    }

    /// The label shown for `mention`.
    pub fn label(&self, mention: &MentionData) -> String {
        (self.label)(mention)
    }
}

impl fmt::Debug for MentionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MentionType")
            .field("trigger", &self.trigger)
            .field("pattern", &self.pattern)
            .field("allowed_spaces", &self.allowed_spaces)
            .field("space_after", &self.space_after)
            .finish_non_exhaustive()
    }
}

/// A type that only styles what it matches, such as bold text or a link.
///
/// The match stays in the value as it is and remains freely editable.
#[derive(Clone, Debug)]
pub struct PatternType {
    pub(crate) pattern: Arc<dyn Pattern>,
}

impl PatternType {
    /// A pattern type matching `pattern`.
    pub fn new(pattern: impl Pattern + 'static) -> Self {
        Self {
            pattern: Arc::new(pattern),
        }
    }
}

/// What to look for in a value: a [`MentionType`] or a [`PatternType`].
///
/// Types are applied in order. Each match becomes one part that's never parsed again, and only the
/// text between matches goes on to the types after it, so styles don't nest.
#[derive(Clone, Debug)]
pub enum PartType {
    /// Turns matches into mentions.
    Mention(MentionType),
    /// Only marks matches, for styling.
    Pattern(PatternType),
}

impl PartType {
    pub(crate) fn pattern(&self) -> &dyn Pattern {
        match self {
            Self::Mention(mention) => &*mention.pattern,
            Self::Pattern(pattern) => &*pattern.pattern,
        }
    }

    /// The mention type, if this is one.
    pub fn as_mention(&self) -> Option<&MentionType> {
        match self {
            Self::Mention(mention) => Some(mention),
            Self::Pattern(_) => None,
        }
    }
}

impl From<MentionType> for PartType {
    fn from(mention: MentionType) -> Self {
        Self::Mention(mention)
    }
}

impl From<PatternType> for PartType {
    fn from(pattern: PatternType) -> Self {
        Self::Pattern(pattern)
    }
}
