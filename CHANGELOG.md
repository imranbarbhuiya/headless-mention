# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-11-13

### Added

- `parse`, splitting a value into mention, pattern and plain text parts.
- `apply_text_change`, turning a plain text edit back into a value while keeping untouched
  mentions.
- `keyword` and `insert_mention`, for showing and applying suggestions.
- Selection mapping between the plain text and the value, through `Parsed::to_value_range` and
  `Parsed::to_plain_range`.
- `MentionState`, wrapping all of the above for one input.
- The `Pattern` trait, implemented for `regex::Regex`, and for `fancy_regex::Regex` behind the
  `fancy-regex` feature.
- UTF-16 offset conversions in `offset`.

[0.1.0]: https://github.com/imranbarbhuiya/headless-mention/releases/tag/v0.1.0
