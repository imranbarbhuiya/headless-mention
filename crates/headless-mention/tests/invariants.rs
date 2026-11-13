//! Randomized edits checked against invariants that must hold for any input.

mod common;

use common::{bold, users};
use headless_mention::{Bias, PartType, apply_text_change, parse};

/// A small deterministic generator, so failures reproduce without extra dependencies.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).unwrap()
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
}

const PIECES: &[&str] = &[
    "a",
    "b",
    " ",
    "\n",
    "@",
    "*",
    "**",
    "é",
    "👋🏻",
    "👨‍👩‍👧‍👦",
    "<@1>",
    "<@22>",
    "<@x>",
];
// Typed text never contains `<` or `>`, so it can't form a new mention.
const TYPED: &[&str] = &["a", "z", " ", "\n", "@", "*", "é", "😀", "1"];

fn random_text(rng: &mut Rng, pieces: &[&str], max: usize) -> String {
    (0..rng.below(max)).map(|_| rng.pick(pieces)).collect()
}

fn char_boundary(rng: &mut Rng, text: &str) -> usize {
    let boundaries: Vec<usize> = text
        .char_indices()
        .map(|(offset, _)| offset)
        .chain([text.len()])
        .collect();
    boundaries[rng.below(boundaries.len())]
}

fn random_edit(rng: &mut Rng, text: &str) -> String {
    let a = char_boundary(rng, text);
    let b = char_boundary(rng, text);
    let (start, end) = (a.min(b), a.max(b));

    format!(
        "{}{}{}",
        &text[..start],
        random_text(rng, TYPED, 4),
        &text[end..]
    )
}

#[test]
fn edits_keep_the_plain_text_and_untouched_mentions() {
    let types: Vec<PartType> = vec![users().into(), bold().into()];
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);

    for _ in 0..5_000 {
        let value = random_text(&mut rng, PIECES, 12);
        let parsed = parse(&value, &types);

        assert_eq!(parsed.value(), value, "parts don't cover {value:?}");

        let edited = random_edit(&mut rng, &parsed.plain_text);
        let new_value = apply_text_change(&parsed, &edited);
        let reparsed = parse(&new_value, &types);

        assert_eq!(
            reparsed.plain_text, edited,
            "{value:?} edited to {edited:?} gave {new_value:?}"
        );
        assert!(
            reparsed.mentions().count() <= parsed.mentions().count(),
            "{value:?} edited to {edited:?} gained a mention"
        );
    }
}

#[test]
fn offsets_round_trip_outside_mentions() {
    let types: Vec<PartType> = vec![users().into()];
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);

    for _ in 0..2_000 {
        let value = random_text(&mut rng, PIECES, 12);
        let parsed = parse(&value, &types);

        for part in parsed.parts.iter().filter(|part| part.mention.is_none()) {
            for (offset, _) in part.text.char_indices() {
                let plain = part.range.start + offset;
                let mapped = parsed.to_value_offset(plain, Bias::Start);

                assert_eq!(parsed.to_plain_offset(mapped, Bias::Start), plain);
            }
        }
    }
}
