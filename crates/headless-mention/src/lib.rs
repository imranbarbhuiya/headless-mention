//! Headless logic for mention inputs, for any text field in any UI toolkit.
//!
//! A value stores mentions as tokens such as `<@42>`, while the user sees and edits labels such as
//! `@Ada`. This crate keeps the two in sync:
//!
//! - [`parse`] splits a value into [`Part`]s: mentions, styled matches such as `**bold**`, and
//!   plain text, along with the plain text the user sees.
//! - [`apply_text_change`] turns an edit of the plain text back into a value, keeping every
//!   untouched mention intact.
//! - [`keyword`] tells you when to show suggestions and what to filter them by, and
//!   [`insert_mention`] applies the one the user picks.
//! - [`Parsed::to_value_range`] and [`Parsed::to_plain_range`] map selections between the two,
//!   for formatting edits.
//! - [`MentionState`] wraps it all up for a single input.
//!
//! It draws nothing, so the field, the styling and the suggestions list are yours.
//!
//! Offsets are byte offsets into UTF-8 text. The [`offset`] module converts to and from UTF-16
//! for input methods and JavaScript.
//!
//! ```
//! use headless_mention::{MentionState, MentionType, PatternType};
//! use regex::Regex;
//!
//! let names = ["Ada", "Grace"];
//! let users = MentionType::new('@', move |mention| {
//!     let name = mention.id.parse::<usize>().ok().and_then(|id| names.get(id));
//!     format!("@{}", name.unwrap_or(&"unknown"))
//! });
//! let bold = PatternType::new(Regex::new(r"\*\*\S(?:.*?\S)?\*\*").unwrap());
//!
//! let mut state = MentionState::new([users.into(), bold.into()]).with_value("**hi** <@0>");
//! assert_eq!(state.plain_text(), "**hi** @Ada");
//!
//! // The user types ` and @gr` at the end.
//! state.edit("**hi** @Ada and @gr", 19..19);
//! assert_eq!(state.keyword("@"), Some("gr"));
//!
//! state.insert_mention("@", "1");
//! assert_eq!(state.value(), "**hi** <@0> and <@1>");
//! assert_eq!(state.plain_text(), "**hi** @Ada and @Grace");
//! ```

mod diff;
mod edit;
pub mod offset;
mod parse;
mod part_type;
mod pattern;
mod selection;
mod state;
mod suggest;

pub use edit::apply_text_change;
pub use parse::{MentionData, Parsed, Part, parse};
pub use part_type::{MentionType, PartType, PatternType};
pub use pattern::{Match, Pattern};
pub use selection::Bias;
pub use state::MentionState;
pub use suggest::{Insertion, insert_mention, keyword};
