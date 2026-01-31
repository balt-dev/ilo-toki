#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Consonant { T = 1, K, P, L, W, S, J, N, M }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Vowel { A, E, I, O, U }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SyllableParseError {
    EmptyString(usize), NoVowel(usize)
}
impl core::fmt::Display for SyllableParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyString(idx) => write!(f, "cannot parse syllable from empty string (index {idx})"),
            Self::NoVowel(idx) => write!(f, "no vowel found after consonant (index {idx})")
        }
    }
}
impl core::error::Error for SyllableParseError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SyllableDecodeError {
    InvalidConsonant(u8), InvalidVowel(u8)
}
impl core::fmt::Display for SyllableDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidConsonant(n) => write!(f, "invalid consonant: {:02X}", n),
            Self::InvalidVowel(n) => write!(f, "invalid vowel: {:02X}", n)
        }
    }
}
impl core::error::Error for SyllableDecodeError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Syllable { pub consonant: Option<Consonant>, pub vowel: Vowel, pub nasal: bool, pub(crate) is_space: bool }

impl core::fmt::Display for Syllable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use {Consonant::*, Vowel::*};
        if self.is_space { return write!(f, "_") }
        match self.consonant {
            None => Ok(()),
            Some(T) => write!(f, "t"), Some(K) => write!(f, "k"), Some(P) => write!(f, "p"),
            Some(L) => write!(f, "l"), Some(W) => write!(f, "w"), Some(S) => write!(f, "s"),
            Some(J) => write!(f, "j"), Some(N) => write!(f, "n"), Some(M) => write!(f, "m"),
        }?;
        match self.vowel {
            A => write!(f, "a"), E => write!(f, "e"), I => write!(f, "i"), O => write!(f, "o"), U => write!(f, "u")
        }?;
        if self.nasal { write!(f, "n")?; }
        Ok(())
    }
}

impl Syllable {
    pub const SPACE: Syllable = Syllable { is_space: true, consonant: None, vowel: Vowel::A, nasal: false };
    
    pub(crate) fn parse(string: &mut &[u8], idx: usize) -> Result<Syllable, SyllableParseError> {
        let mut chars = (*string).iter();
        let mut first = chars.next().ok_or(SyllableParseError::EmptyString(idx))?;
        let consonant = match first {
            b'.' | b',' | b':' | b';' | b'!' | b'?' => { *string = chars.as_slice(); return Ok(Self::SPACE) },
            b't' | b'T' => Some(Consonant::T), b'k' | b'K' => Some(Consonant::K), b'p' | b'P' => Some(Consonant::P),
            b'j' | b'J' => Some(Consonant::J), b'w' | b'W' => Some(Consonant::W), b'l' | b'L' => Some(Consonant::L),
            b's' | b'S' => Some(Consonant::S), b'n' | b'N' => Some(Consonant::N), b'm' | b'M' => Some(Consonant::M),
            _ => None
        };
        if consonant.is_some() { first = chars.next().ok_or(SyllableParseError::NoVowel(idx))?; }
        let vowel = match first {
            b'a' | b'A' => Vowel::A, b'e' | b'E' => Vowel::E, b'i' | b'I' => Vowel::I,
            b'o' | b'O' => Vowel::O, b'u' | b'U' => Vowel::U,
            _ => return Err(SyllableParseError::NoVowel(idx))
        };
        let mut nasal = false;
        if chars.as_slice().starts_with(b"n") {
            let next = chars.as_slice().iter().skip(1).next();
            if !next.is_some_and(|c| matches!(c, b'a'|b'e'|b'i'|b'o'|b'u')) {
                chars.next();
                nasal = true;
            }
        }
        *string = chars.as_slice();
        return Ok(Syllable { consonant, vowel, nasal, is_space: false })
    }

    pub const fn encode(&self) -> u8 {
        if self.is_space { return 0xFF };
        let cons = if let Some(c) = self.consonant {c as u8} else {0};
        cons << 4 | (self.vowel as u8) << 1 | self.nasal as u8
    }
    pub const fn decode(byte: u8) -> Result<Self, SyllableDecodeError> {
        if byte == 0xFF { return Ok(Self::SPACE) }
        use {Consonant::*, Vowel::*};
        let consonant = match (byte & 0b11110000) >> 4 {
            0 => None,
            1 => Some(T), 2 => Some(K), 3 => Some(P),
            4 => Some(L), 5 => Some(W), 6 => Some(S),
            7 => Some(J), 8 => Some(N), 9 => Some(M),
            n => return Err(SyllableDecodeError::InvalidConsonant(n))
        };
        let vowel = match (byte & 0b00001110) >> 1 {
            0 => A, 1 => E, 2 => I, 3 => O, 4 => U,
            n => return Err(SyllableDecodeError::InvalidVowel(n))
        };
        let nasal = byte & 0b00000001 != 0;
        return Ok(Self { consonant, vowel, nasal, is_space: false })
    }
}

pub fn parse_syllables<'str> (string: &'str [u8]) -> impl Iterator<Item = Option<Result<Syllable, SyllableParseError>>> + 'str {
    let mut s = string.as_ref();
    let initial_len = string.len();
    let mut done = false;
    core::iter::from_fn(move || {
        if done { return None; }
        let Some(first) = s.first() else { done = true; return Some(None); };
        if first.is_ascii_whitespace() {
            s = s.trim_ascii_start();
            return Some(None)
        }
        let len = s.len();
        let res = Syllable::parse(&mut s, initial_len - len);
        if res.is_err() { done = true; }
        Some(Some(res))
    })
}
