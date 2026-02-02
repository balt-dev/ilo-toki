use std::io::{Cursor, Seek, SeekFrom, Write};

use wasm_bindgen::prelude::*;
use ilo_toki::{SynthSettings, parse_segments, pronounce_segments};

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
}

#[wasm_bindgen]
pub fn generate(
    value: &str,
    sample_rate: u32,
    vowel_time: f32,
    consonant_time: f32,
    space_time: f32,
    base_frequency: f32,
) -> JsValue {
    let settings = SynthSettings { base_frequency, sample_rate, vowel_time, consonant_time, space_time };
    let syllables = match parse_segments(value.as_bytes(), settings).collect::<Result<Vec<_>, _>>() {
        Ok(v) => v,
        Err(err) => {return format!("[ERROR]\n{err}").into()}
    };
    let pronounced = pronounce_segments(syllables.into_iter().inspect(|syl| {
        log(format!("Syllable: {syl:?}"))
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