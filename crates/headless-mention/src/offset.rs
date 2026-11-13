//! Conversions between byte offsets and UTF-16 offsets.
//!
//! This crate works in byte offsets, but many text systems count UTF-16 code units, including
//! platform input methods and JavaScript. Use these at that boundary.

/// The byte offset of `utf16` UTF-16 code units into `text`.
///
/// An offset inside a surrogate pair rounds down to the start of its char, and one past the end
/// returns `text.len()`.
pub fn utf16_to_byte(text: &str, utf16: usize) -> usize {
    let mut units = 0;

    for (offset, char) in text.char_indices() {
        let next = units + char.len_utf16();
        if next > utf16 {
            return offset;
        }
        units = next;
    }

    text.len()
}

/// The number of UTF-16 code units before byte `offset` of `text`.
///
/// An offset inside a char counts up to the start of that char, and one past the end counts all
/// of `text`.
pub fn byte_to_utf16(text: &str, offset: usize) -> usize {
    text.char_indices()
        .take_while(|(start, char)| start + char.len_utf8() <= offset)
        .map(|(_, char)| char.len_utf16())
        .sum()
}
