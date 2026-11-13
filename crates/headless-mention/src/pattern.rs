use std::{fmt, ops::Range};

/// One match of a [`Pattern`].
///
/// Every range is a byte range into the haystack the pattern was run on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    /// The whole match.
    pub range: Range<usize>,
    /// The `trigger` group, only read for mention types.
    pub trigger: Option<Range<usize>>,
    /// The `id` group, only read for mention types.
    pub id: Option<Range<usize>>,
}

/// A regex-like matcher that finds mentions or styled runs in a value.
///
/// It's implemented for [`regex::Regex`] and, with the `fancy-regex` feature, for
/// `fancy_regex::Regex`, which adds look-around and backreferences. Both read the `trigger` and `id`
/// groups by name, falling back to groups 1 and 2 when the pattern has no named groups.
///
/// Implement it yourself to plug in any other engine.
pub trait Pattern: fmt::Debug + Send + Sync {
    /// Calls `on_match` for every non-overlapping match in `haystack`, in order.
    fn for_each_match(&self, haystack: &str, on_match: &mut dyn FnMut(Match));
}

impl Pattern for regex::Regex {
    fn for_each_match(&self, haystack: &str, on_match: &mut dyn FnMut(Match)) {
        for captures in self.captures_iter(haystack) {
            let group = |name: &str, index: usize| {
                captures
                    .name(name)
                    .or_else(|| captures.get(index))
                    .map(|m| m.range())
            };

            on_match(Match {
                range: captures.get(0).expect("group 0 is the whole match").range(),
                trigger: group("trigger", 1),
                id: group("id", 2),
            });
        }
    }
}

#[cfg(feature = "fancy-regex")]
impl Pattern for fancy_regex::Regex {
    fn for_each_match(&self, haystack: &str, on_match: &mut dyn FnMut(Match)) {
        // An error here means the backtracking limit was hit, so the rest of the value stays plain text.
        for captures in self.captures_iter(haystack).map_while(Result::ok) {
            let group = |name: &str, index: usize| {
                captures
                    .name(name)
                    .or_else(|| captures.get(index))
                    .map(|m| m.start()..m.end())
            };

            let whole = captures.get(0).expect("group 0 is the whole match");

            on_match(Match {
                range: whole.start()..whole.end(),
                trigger: group("trigger", 1),
                id: group("id", 2),
            });
        }
    }
}
