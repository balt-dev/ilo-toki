#![no_std]

pub(crate) mod speak;
pub(crate) mod syl;

pub use speak::{SynthSettings, pronounce_segments};
pub use syl::{SoundSegment, SegmentParseError, Vowel, Consonant, parse_segments};

#[doc(hidden)]
pub use speak::pronounce_single;