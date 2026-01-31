use std::io::{Cursor, Seek, SeekFrom, Write};

use wasm_bindgen::prelude::*;
use ilo_toki::{SynthSettings, parse_syllables, pronounce_syllables};

static WAV_HEADER: [u8; 44] = [
    0x52, 0x49, 0x46, 0x46, 0xFF, 0xFF, 0xFF, 0xFF,
    0x57, 0x41, 0x56, 0x45, 0x66, 0x6D, 0x74, 0x20,
    0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x10, 0x00, 0x64, 0x61, 0x74, 0x61,
    0xFF, 0xFF, 0xFF, 0xFF,
];

#[wasm_bindgen]
pub fn generate(
    value: &str,
    sample_rate: u32,
    vowel_time: f32,
    consonant_time: f32,
    space_time: f32,
    pitch: f32,
) -> JsValue {
    let syllables = match parse_syllables(value.as_bytes()).map(|v| v.transpose()).collect::<Result<Vec<_>, _>>() {
        Ok(v) => v,
        Err(err) => {return format!("[ERROR]\n{err}").into()}
    };
    let pronounced = pronounce_syllables(syllables.into_iter(), SynthSettings { pitch, sample_rate, vowel_time, consonant_time, space_time })
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