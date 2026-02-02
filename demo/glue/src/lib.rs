use std::io::{Cursor, Seek, SeekFrom, Write};

use wasm_bindgen::prelude::*;
use ilo_toki::{Consonant, SoundSegment, SoundSegmentKind, SynthSettings, Vowel, pronounce_segments};

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

pub fn parse<'settings, 'str: 'settings>(string: &mut &'str [u8], settings: &'settings mut SynthSettings) -> Result<SoundSegment, SegmentParseError> {
    let mut chars = (*string).iter();
    'a: { 'b: { if string.first().is_some_and(|c| *c == b'<') {
        let slice = chars.as_slice();
        let Some(settings_end) = chars.position(|c| *c == b'>') else { break 'b; };
        let settings_str = &slice[1..settings_end];
        let Some(space_idx) = settings_str.iter().position(|c| *c == b' ') else { break 'b; };
        let mut len_str = settings_str;
        let Some(mut freq_str) = len_str.split_off(space_idx..) else { break 'b; };
        len_str = len_str.trim_ascii();
        freq_str = freq_str.trim_ascii();
        let Some(len) = str::from_utf8(len_str).ok().and_then(|v| v.parse::<f32>().ok()) else { break 'b; };
        let Some(freq) = str::from_utf8(freq_str).ok().and_then(|v| v.parse::<f32>().ok()) else { break 'b; };
        settings.space_time *= len;
        settings.consonant_time *= len;
        settings.vowel_time *= len;
        settings.base_frequency = freq;
        *string = chars.as_slice();
        break 'a;
    } } chars = (*string).iter(); }
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
    let length = match kind {
        SoundSegmentKind::Space => settings.space_time,
        SoundSegmentKind::Consonant(_) => settings.consonant_time,
        SoundSegmentKind::Vowel(_) => settings.vowel_time
    };
    let frequency = settings.base_frequency;
    Ok(SoundSegment { kind, length, frequency })
}

#[doc(hidden)]
pub fn parse_segments<'settings, 'str: 'settings> (string: &'str [u8], mut settings: SynthSettings) -> Result<Vec<SoundSegment>, SegmentParseError> {
    let mut s = string.as_ref();
    let full_len = string.len();
    let mut done = false;
    let mut res = core::iter::from_fn(move || {
        if done { return None; }
        let mut res = parse(&mut s, &mut settings);
        if let Err(err) = &mut res {
            done = true;
            match err {
                SegmentParseError::EmptyString(_) => return None,
                SegmentParseError::Unmatched(idx, _) => *idx += full_len - s.len()
            }
        }
        Some(res)
    }).collect::<Result<Vec<_>, _>>()?;
    
    for i in 0..res.len() {
        let last = i.checked_sub(1).and_then(|i| res.get(i).copied());
        let next = i.checked_add(1).and_then(|i| res.get(i).copied());
        let next_next = i.checked_add(2).and_then(|i| res.get(i).copied());
        let cur = &mut res[i];
        if let SoundSegmentKind::Vowel(_) = cur.kind {
            if let Some(SoundSegmentKind::Consonant(_)) = last.map(|c| c.kind) {
                cur.length -= settings.consonant_time;
            }
            if let Some(SoundSegmentKind::Consonant(Consonant::N)) = next.map(|c| c.kind) {
                if let Some(SoundSegmentKind::Vowel(_)) = next_next.map(|c| c.kind) {}
                else { cur.length -= settings.consonant_time; }
            }
        }
    }
    
    Ok(res)
}

static WAV_HEADER: [u8; 44] = [
    0x52, 0x49, 0x46, 0x46, 0xFF, 0xFF, 0xFF, 0xFF,
    0x57, 0x41, 0x56, 0x45, 0x66, 0x6D, 0x74, 0x20,
    0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x10, 0x00, 0x64, 0x61, 0x74, 0x61,
    0xFF, 0xFF, 0xFF, 0xFF,
];

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: String);
    #[wasm_bindgen(js_namespace = console)]
    fn clear();
}

#[wasm_bindgen]
pub fn generate(
    value: &str,
    sample_rate: u32,
    vowel_time: f32,
    consonant_time: f32,
    space_time: f32,
    base_frequency: f32,
    transition_time: f32
) -> JsValue {
    if vowel_time < consonant_time * 2.0 {
        return format!(concat!(
            "[ERROR]\nVowel time must be at least twice the consonant time.\n",
            "(a -> {:.02}, la -> {:.02}, lan -> {:.02})"
        ), vowel_time, vowel_time - consonant_time, vowel_time - consonant_time * 2.0).into();
    }
    let settings = SynthSettings { base_frequency, sample_rate, vowel_time, consonant_time, space_time, transition_time };
    let syllables = match parse_segments(value.as_bytes(), settings) {
        Ok(v) => v,
        Err(err) => {return format!("[ERROR]\n{err}").into()}
    };
    clear();
    let pronounced = pronounce_segments(syllables.into_iter().inspect(|syl| {
        log(format!("{syl}"))
    }), settings)
        .map(|v| (v.clamp(-1.0, 1.0) * (i16::MAX as f32)) as i16);
    let mut file = Cursor::new(Vec::<u8>::new());
    #[allow(unused_must_use)] {
        file.write_all(&WAV_HEADER);
        file.seek(SeekFrom::Start(24));
        file.write_all(&sample_rate.to_le_bytes());
        file.write_all(&(sample_rate.wrapping_mul(2)).to_le_bytes());
        file.seek(SeekFrom::End(0));
        for sample in pronounced {
            file.write_all(&sample.to_le_bytes());
        }
    }
    let data = file.into_inner();
    data.into()
}

#[wasm_bindgen]
pub fn debug_generate(f1: f32, f2: f32, f3: f32, b1: f32, b2: f32, b3: f32, pitch: f32, sample_rate: u32) -> Vec<u8> {
    let mut file = Cursor::new(Vec::<u8>::new());
    #[allow(unused_must_use)] {
        file.write_all(&WAV_HEADER);
        file.seek(SeekFrom::Start(24));
        file.write_all(&sample_rate.to_le_bytes());
        file.write_all(&(sample_rate.wrapping_mul(2)).to_le_bytes());
        file.seek(SeekFrom::End(0));
        let samples = ilo_toki::pronounce_single([f1, f2, f3], [b1, b2, b3], pitch, 0.5, sample_rate);
        for sample in samples {
            let u16_sample = (sample.clamp(-1.0, 1.0) * (i16::MAX as f32)) as i16;
            file.write_all(&u16_sample.to_le_bytes());
        }
    }
    let data = file.into_inner();
    data.into()
}