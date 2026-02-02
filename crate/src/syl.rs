use crate::SynthSettings;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Consonant { T, K, P, L, W, S, J, N, M }

impl core::fmt::Display for Consonant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Consonant::*;
        match self {
            T => write!(f, "t"), K => write!(f, "k"), P => write!(f, "p"),
            L => write!(f, "l"), W => write!(f, "w"), S => write!(f, "s"),
            J => write!(f, "j"), N => write!(f, "n"), M => write!(f, "m"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Vowel { A, E, I, O, U }

impl core::fmt::Display for Vowel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Vowel::*;
        match self {
            A => write!(f, "a"), E => write!(f, "e"), I => write!(f, "i"), O => write!(f, "o"), U => write!(f, "u")
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SegmentParseError {
    EmptyString(usize),
    Unmatched(usize, u8)
}
impl core::fmt::Display for SegmentParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyString(idx) => write!(f, "at index {idx}: cannot parse sound segment from empty string"),
            Self::Unmatched(idx, chr @ 0x20 .. 0x7F) => write!(f, "at index {idx}: not a sound segment: '{}'", char::from_u32(*chr as u32).unwrap_or('\0')),
            Self::Unmatched(idx, chr) => write!(f, "at index {idx}: not a sound segment: \\x{:02X}", chr)
        }
    }
}
impl core::error::Error for SegmentParseError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SoundSegmentKind {
    Vowel(Vowel),
    Consonant(Consonant),
    Space
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SoundSegment {
    pub kind: SoundSegmentKind,
    pub length: f32,
    pub frequency: f32
}

impl core::fmt::Display for SoundSegment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            SoundSegmentKind::Vowel(vowel) => write!(f, "{vowel}"),
            SoundSegmentKind::Consonant(consonant) => write!(f, "{consonant}"),
            SoundSegmentKind::Space => write!(f, "_")
        }?;
        write!(f, "<{:.02} {:.02}>", self.length, self.frequency)?;
        Ok(())
    }
}

impl SoundSegment {
    pub(crate) fn parse<'settings, 'str: 'settings>(string: &mut &'str [u8], settings: &'settings mut SynthSettings) -> Result<SoundSegment, SegmentParseError> {
        let mut chars = (*string).iter();
        let first = chars.next().ok_or(SegmentParseError::EmptyString(0))?;
        let kind = match first {
            b'a' | b'A' => SoundSegmentKind::Vowel(Vowel::A),
            b'e' | b'E' => SoundSegmentKind::Vowel(Vowel::E),
            b'i' | b'I' => SoundSegmentKind::Vowel(Vowel::I),
            b'o' | b'O' => SoundSegmentKind::Vowel(Vowel::O),
            b'u' | b'U' => SoundSegmentKind::Vowel(Vowel::U),
            b't' | b'T' | b'd' | b'D' => SoundSegmentKind::Consonant(Consonant::T),
            b'p' | b'P' | b'b' | b'B' | b'f' | b'F' => SoundSegmentKind::Consonant(Consonant::P),
            b'k' | b'K' | b'c' | b'C' | b'g' | b'G' | b'q' | b'Q' => SoundSegmentKind::Consonant(Consonant::K),
            b'l' | b'L' => SoundSegmentKind::Consonant(Consonant::L),
            b'w' | b'W' | b'v' | b'V' | b'r' | b'R' => SoundSegmentKind::Consonant(Consonant::W),
            b'j' | b'J' | b'y' | b'Y' => SoundSegmentKind::Consonant(Consonant::J),
            b's' | b'S' | b'x' | b'X' | b'z' | b'Z' => SoundSegmentKind::Consonant(Consonant::S),
            b'n' | b'N' => SoundSegmentKind::Consonant(Consonant::N),
            b'm' | b'M' => SoundSegmentKind::Consonant(Consonant::M),
            b'.' | b',' | b'-' | b'/' | b':' | b';' | b'?' | b'!' |
            b' ' | b'\t' | b'\n' | b'\r' | b'(' | b')' | b'"' | b'\'' |
            b'h' | b'H'
                => SoundSegmentKind::Space,
            c => return Err(SegmentParseError::Unmatched(0, *c))
        };
        *string = chars.as_slice();
        if kind == SoundSegmentKind::Space {
            *string = string.trim_ascii_start();
        }
        let mut length = match kind {
            SoundSegmentKind::Space => settings.space_time,
            SoundSegmentKind::Consonant(_) => settings.consonant_time,
            SoundSegmentKind::Vowel(_) => settings.vowel_time
        };
        let mut frequency = settings.base_frequency;
        'b: { if chars.next().is_some_and(|c| *c == b'<') {
            let slice = chars.as_slice();
            let Some(settings_end) = chars.position(|c| *c == b'>') else { break 'b; };
            let settings_str = &slice[..settings_end];
            let Some(space_idx) = settings_str.iter().position(|c| *c == b' ') else { break 'b; };
            let mut len_str = settings_str;
            let Some(mut freq_str) = len_str.split_off(space_idx..) else { break 'b; };
            len_str = len_str.trim_ascii();
            freq_str = freq_str.trim_ascii();
            let Some(len) = str::from_utf8(len_str).ok().and_then(|v| v.parse::<f32>().ok()) else { break 'b; };
            let Some(freq) = str::from_utf8(freq_str).ok().and_then(|v| v.parse::<f32>().ok()) else { break 'b; };
            if !(length.is_finite() && frequency.is_finite()) { break 'b; }
            length /= len; frequency *= freq;
            settings.space_time /= len;
            settings.consonant_time /= len;
            settings.vowel_time /= len;
            settings.base_frequency = frequency;
            *string = chars.as_slice();
        } }
        Ok(SoundSegment { kind, length, frequency })
    }
}

pub fn parse_segments<'settings, 'str: 'settings> (string: &'str [u8], mut settings: SynthSettings) -> impl Iterator<Item = Result<SoundSegment, SegmentParseError>> {
    let mut s = string.as_ref();
    let full_len = string.len();
    let mut done = false;
    core::iter::from_fn(move || {
        if done { return None; }
        let mut res = SoundSegment::parse(&mut s, &mut settings);
        if let Err(err) = &mut res {
            done = true;
            match err {
                SegmentParseError::EmptyString(_) => return None,
                SegmentParseError::Unmatched(idx, _) => *idx += full_len - s.len()
            }
        }
        Some(res)
    })
}

#[test]
fn test_parsing() {
    let settings = SynthSettings::default();
    let segs = parse_segments(b"kijetesantakalu?? li tawa sike??? monsuta....", settings);
    for seg in segs {
        seg.unwrap();
    }
    let mut segs = parse_segments(b"# gaming !!", settings);
    assert!(segs.next().is_some_and(|e| e.is_err()));
}