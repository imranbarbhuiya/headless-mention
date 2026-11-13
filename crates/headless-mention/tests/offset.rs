//! Byte and UTF-16 offset conversions.

use headless_mention::offset::{byte_to_utf16, utf16_to_byte};

const TEXT: &str = "a😀é";

#[test]
fn utf16_to_byte_offsets() {
    assert_eq!(utf16_to_byte(TEXT, 0), 0);
    assert_eq!(utf16_to_byte(TEXT, 1), 1);
    assert_eq!(utf16_to_byte(TEXT, 3), 5);
    assert_eq!(utf16_to_byte(TEXT, 4), 7);
}

#[test]
fn utf16_inside_a_surrogate_pair_rounds_down() {
    assert_eq!(utf16_to_byte(TEXT, 2), 1);
}

#[test]
fn utf16_past_the_end_clamps() {
    assert_eq!(utf16_to_byte(TEXT, 100), TEXT.len());
}

#[test]
fn byte_to_utf16_offsets() {
    assert_eq!(byte_to_utf16(TEXT, 0), 0);
    assert_eq!(byte_to_utf16(TEXT, 1), 1);
    assert_eq!(byte_to_utf16(TEXT, 5), 3);
    assert_eq!(byte_to_utf16(TEXT, 7), 4);
}

#[test]
fn byte_inside_a_char_rounds_down() {
    assert_eq!(byte_to_utf16(TEXT, 3), 1);
    assert_eq!(byte_to_utf16(TEXT, 100), 4);
}

#[test]
fn round_trips_on_char_boundaries() {
    for (offset, _) in TEXT.char_indices() {
        assert_eq!(utf16_to_byte(TEXT, byte_to_utf16(TEXT, offset)), offset);
    }
}
