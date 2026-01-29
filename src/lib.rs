pub(crate) mod speak;
pub(crate) mod syl;

pub use speak::{SynthSettings, pronounce_syllables};
pub use syl::{Syllable, SyllableDecodeError, SyllableParseError, Vowel, Consonant, parse_syllables};