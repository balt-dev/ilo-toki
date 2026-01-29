#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Consonant { T = 1, K, P, L, W, S, J, N, M }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Vowel { A, E, I, O, U }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SyllableParseError {
    EmptyString, NoVowel(usize)
}
impl std::fmt::Display for SyllableParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyString => write!(f, "cannot parse syllable from empty string"),
            Self::NoVowel(idx) => write!(f, "no vowel found after consonant (index {idx})")
        }
    }
}
impl std::error::Error for SyllableParseError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SyllableDecodeError {
    InvalidConsonant(u8), InvalidVowel(u8)
}
impl std::fmt::Display for SyllableDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConsonant(n) => write!(f, "invalid consonant: {:02X}", n),
            Self::InvalidVowel(n) => write!(f, "invalid vowel: {:02X}", n)
        }
    }
}
impl std::error::Error for SyllableDecodeError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Syllable { pub consonant: Option<Consonant>, pub vowel: Vowel, pub nasal: bool, pub(crate) is_space: bool }

impl std::fmt::Display for Syllable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        let mut first = chars.next().ok_or(SyllableParseError::EmptyString)?;
        let consonant = match first {
            b'.' | b',' | b':' | b';' | b'!' | b'?' => { *string = chars.as_slice(); return Ok(Self::SPACE) },
            b't' => Some(Consonant::T), b'k' => Some(Consonant::K), b'p' => Some(Consonant::P),
            b'j' => Some(Consonant::J), b'w' => Some(Consonant::W), b'l' => Some(Consonant::L),
            b's' => Some(Consonant::S), b'n' => Some(Consonant::N), b'm' => Some(Consonant::M),
            _ => None
        };
        if consonant.is_some() { first = chars.next().ok_or(SyllableParseError::NoVowel(idx))?; }
        let vowel = match first {
            b'a' => Vowel::A, b'e' => Vowel::E, b'i' => Vowel::I,
            b'o' => Vowel::O, b'u' => Vowel::U,
            _ => return Err(if consonant.is_some() {
                    SyllableParseError::NoVowel(idx)
                } else {
                    SyllableParseError::EmptyString
                })
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
    std::iter::from_fn(move || {
        if s.first()?.is_ascii_whitespace() {
            s = s.trim_ascii_start();
            return Some(None)
        }
        let len = s.len();
        Some(Some(Syllable::parse(&mut s, initial_len - len)))
    })
}

#[test]
fn test_encode_decode() {
    for c in 0..=9 {
        for v in 0..=4 {
            for n in 0..=1 {
                let byte = (c << 4) | (v << 1) | n;
                let syl = Syllable::decode(byte).unwrap();
                eprintln!("{syl}");
                assert_eq!(syl.consonant.map_or(0, |x| x as u8), c);
                assert_eq!(syl.vowel as u8, v);
                assert_eq!(syl.nasal as u8, n);
                assert_eq!(syl.encode(), byte);
            }
        }
    }
}
