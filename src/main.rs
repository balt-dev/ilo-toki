use ilo_toki::{SynthSettings, parse_syllables, pronounce_syllables};

use std::{env::args, fs::File, io::{BufWriter, Cursor, Read, SeekFrom, Write, stdin}, process::ExitCode};

macro_rules! try_or_exit {
    ($expr: expr; $err: pat => $block: block) => {
        match $expr {
            Ok(v) => v,
            Err($err) => return $block
        }
    };
}

fn main() -> ExitCode {
    let mut args = args();
    let Some(_prog_name) = args.next() else { eprintln!("first argument should be program name, did not exist"); return ExitCode::FAILURE; };
    let Some(filepath) = args.next() else {
        eprintln!("Usage: ilo_toki <output.wav> [sample rate = 44100] [vowel time = 0.4] [consonant time = vowel time / 4.0]\nGets an input toki pona string from stdin and synthesizes it.");
        return ExitCode::SUCCESS;
    };
    let sample_rate = try_or_exit! {
        args.next().map_or(Ok(44100), |v| v.parse::<u32>());
        _ => { eprintln!("Failed to parse sample rate"); ExitCode::FAILURE }
    };
    let vowel_time = try_or_exit! {
        args.next().map_or(Ok(0.4), |v| v.parse::<f32>());
        _ => { eprintln!("Failed to parse vowel time"); ExitCode::FAILURE }
    };
    let consonant_time = try_or_exit! {
        args.next().map_or(Ok(vowel_time / 4.0), |v| v.parse::<f32>());
        _ => { eprintln!("Failed to parse consonant time"); ExitCode::FAILURE }
    };
    let mut string = Vec::new();
    try_or_exit! { 
        stdin().read_to_end(&mut string); 
        err => { eprintln!("I/O error when reading stdin: {err}"); ExitCode::FAILURE }
    };
    let syllables = try_or_exit! { 
        parse_syllables(&string).map(|v| v.transpose()).collect::<Result<Vec<_>, _>>();
        err => { eprintln!("Failed to parse input string: {err}"); ExitCode::FAILURE }
    };
    eprintln!("Syllables: {syllables:?}");
    let settings = SynthSettings { sample_rate, consonant_time, vowel_time };
    
    let file = try_or_exit! {
        File::create(filepath).map(BufWriter::new);
        err => { eprintln!("Failed to open output file: {err}"); ExitCode::FAILURE }
    };
    
    try_or_exit! {
        write_wav(pronounce_syllables(syllables.into_iter(), settings), sample_rate, file);
        err => { eprintln!("Failed to write output file: {err}"); ExitCode::FAILURE }
    }

    ExitCode::SUCCESS
}

fn write_wav(iter: impl Iterator<Item = f32>, sample_rate: u32, mut file: impl std::io::Write + std::io::Seek) -> Result<(), std::io::Error> {
    file.write_all(b"RIFF")?;
    let size_idx = file.stream_position()? as u32;
    file.write_all(&[0u8; 4])?;
    file.write_all(b"WAVE")?;
    
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // subchunk
    file.write_all(&1u16.to_le_bytes())?;  // pcm
    file.write_all(&1u16.to_le_bytes())?;  // channel count
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&(sample_rate * 2).to_le_bytes())?; // byte rate
    file.write_all(&2u16.to_le_bytes())?;  // alignment
    file.write_all(&16u16.to_le_bytes())?; // bits per sample
    
    file.write_all(b"data")?;
    let data_idx = file.stream_position()? as u32;
    file.write_all(&[0u8; 4])?;
    
    for sample in iter {
        let quantized: i16 = ((sample * 0.4).clamp(-1.0, 1.0) * (i16::MAX as f32)) as i16;
        let bytes = quantized.to_le_bytes();
        file.write_all(&bytes)?;
    }
    
    let end_pos = file.stream_position()? as u32;
    let data_size = end_pos - data_idx - 4;
    file.seek(SeekFrom::Start(data_idx as u64))?;
    file.write_all(&data_size.to_le_bytes())?;
    file.seek(SeekFrom::Start(size_idx as u64))?;
    file.write_all(&end_pos.to_le_bytes())?;
    file.seek(SeekFrom::End(0))?;
    
    Ok(())
}