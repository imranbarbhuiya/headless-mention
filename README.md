# headless-mention

[![crates.io](https://img.shields.io/crates/v/headless-mention.svg)](https://crates.io/crates/headless-mention)
[![docs.rs](https://img.shields.io/docsrs/headless-mention)](https://docs.rs/headless-mention)
[![CI](https://github.com/imranbarbhuiya/headless-mention/actions/workflows/ci.yml/badge.svg)](https://github.com/imranbarbhuiya/headless-mention/actions/workflows/ci.yml)

Mention input logic for any Rust UI toolkit. It draws nothing: you bring the text field, the
styling and the suggestions list, and it keeps the value consistent.

A value stores mentions as tokens such as `<@42>`, while the user sees and edits labels such as
`@Ada`. This crate:

- splits a value into parts (mentions, styled matches such as `**bold**`, and plain text) along
  with the plain text the user sees;
- turns every edit of the plain text back into a value, keeping untouched mentions intact and
  turning an edited label into plain text;
- tells you when to show suggestions and what to filter them by, and inserts the one the user
  picks;
- maps selections between the plain text and the value, for formatting edits such as wrapping the
  selection in `**`.

It's the Rust counterpart of
[`react-native-headless-mention`](https://github.com/imranbarbhuiya/react-native-headless-mention).

## Install

```sh
cargo add headless-mention
```

## Usage

```rust
use headless_mention::{MentionState, MentionType, PatternType};
use regex::Regex;

let users = MentionType::new('@', |mention| format!("@{}", lookup_name(&mention.id)))
    .space_after_mention(true);
let bold = PatternType::new(Regex::new(r"\*\*\S(?:.*?\S)?\*\*").unwrap());

let mut state = MentionState::new([users.into(), bold.into()]).with_value(draft);
```

Then wire it to your text field:

1. Show `state.plain_text()` in the field. Style each of `state.parts()` by its `part_type`, the
   index of the type that matched it.
2. On every edit, call `state.edit(new_text, new_selection)`. When it returns `true`, write
   `state.plain_text()` and `state.selection()` back to the field.
3. While `state.active_keyword()` returns `Some((trigger, keyword))`, show suggestions filtered by
   `keyword`. When one is picked, call `state.insert_mention(trigger, id)` and write the text and
   selection back.
4. Store or send `state.value()`. Clear the field with `state.set_value("")`.

`MentionState` is a thin wrapper. The functions it uses (`parse`, `apply_text_change`, `keyword`,
`insert_mention`) are public too, for when you'd rather hold the state yourself.

### Offsets

Every offset is a byte offset into UTF-8 text. Input methods and JavaScript count UTF-16 code units
instead, so convert at that boundary with `headless_mention::offset::{utf16_to_byte, byte_to_utf16}`.

### Patterns

Mention and pattern types take anything implementing `Pattern`. It's implemented for
`regex::Regex` and, with the `fancy-regex` feature, for `fancy_regex::Regex`, which supports
look-around:

```toml
headless-mention = { version = "0.1", features = ["fancy-regex"] }
```

A mention pattern needs a `trigger` and an `id` group, named or as groups 1 and 2. The default
matches `<{trigger}{id}>`. Use `MentionType::pattern` for another format.

## Behaviour to know

- Part types run in order, and a match is never parsed again, so styles don't nest. Only the text
  between matches goes on to the later types.
- Patterns see the whole value, or the slice between earlier matches. Rules such as "emphasis
  doesn't cross a line break" belong in the pattern.
- Suggestions only start at a trigger that begins the text or follows whitespace. A new line ends
  the keyword, and so do more spaces than `MentionType::allowed_spaces` (1 by default).

## License

[MIT](LICENSE)
