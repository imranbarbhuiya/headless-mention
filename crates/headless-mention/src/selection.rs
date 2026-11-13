use std::ops::Range;

use crate::Parsed;

/// Which edge of a mention an offset inside its label snaps to, since a mention can't be split.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bias {
    /// Snap to the start of the mention.
    Start,
    /// Snap to the end of the mention.
    End,
}

impl Parsed {
    /// Maps a byte offset in the plain text to one in the value.
    ///
    /// An offset past the end maps to the end of the value.
    pub fn to_value_offset(&self, offset: usize, bias: Bias) -> usize {
        map_offset(self, offset, bias, |part| {
            (part.range.clone(), part.value_range.clone())
        })
    }

    /// Maps a byte offset in the value to one in the plain text.
    ///
    /// An offset past the end maps to the end of the plain text.
    pub fn to_plain_offset(&self, offset: usize, bias: Bias) -> usize {
        map_offset(self, offset, bias, |part| {
            (part.value_range.clone(), part.range.clone())
        })
    }

    /// Maps a selection in the plain text to the value.
    ///
    /// A selection partly covering a mention grows to cover all of it, and a caret inside one moves
    /// past it. Use it to run a formatting edit, such as wrapping the selection in `**`, on the value.
    pub fn to_value_range(&self, selection: Range<usize>) -> Range<usize> {
        map_range(selection, |offset, bias| self.to_value_offset(offset, bias))
    }

    /// Maps a selection in the value to the plain text, growing it the same way as
    /// [`Parsed::to_value_range`].
    pub fn to_plain_range(&self, selection: Range<usize>) -> Range<usize> {
        map_range(selection, |offset, bias| self.to_plain_offset(offset, bias))
    }
}

fn map_range(selection: Range<usize>, map: impl Fn(usize, Bias) -> usize) -> Range<usize> {
    if selection.is_empty() {
        let caret = map(selection.start, Bias::End);
        return caret..caret;
    }

    map(selection.start, Bias::Start)..map(selection.end, Bias::End)
}

fn map_offset(
    parsed: &Parsed,
    offset: usize,
    bias: Bias,
    ranges: impl Fn(&crate::Part) -> (Range<usize>, Range<usize>),
) -> usize {
    let Some(part) = parsed
        .parts
        .iter()
        .find(|part| offset <= ranges(part).0.end)
    else {
        return parsed.parts.last().map_or(0, |part| ranges(part).1.end);
    };

    let (from, to) = ranges(part);

    if part.mention.is_none() {
        return to.start + (offset - from.start);
    }

    if offset == from.start {
        to.start
    } else if offset == from.end {
        to.end
    } else {
        match bias {
            Bias::Start => to.start,
            Bias::End => to.end,
        }
    }
}
