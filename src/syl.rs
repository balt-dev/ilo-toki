#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Consonant { T = 1, K, P, L, W, S, J, N, M }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Vowel { A, E, I, O, U }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SyllableParseError {
    EmptyString, NoVowel
}
impl std::fmt::Display for SyllableParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyString => write!(f, "cannot parse syllable from empty string"),
            Self::NoVowel => write!(f, "no vowel found after consonant")
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
pub struct Syllable { pub consonant: Option<Consonant>, pub vowel: Vowel, pub nasal: bool }

impl std::fmt::Display for Syllable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use {Consonant::*, Vowel::*};
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
    pub fn parse(string: &mut &str) -> Result<Syllable, SyllableParseError> {
        let mut chars = (*string).chars();
        let mut first = chars.next().ok_or(SyllableParseError::EmptyString)?;
        let consonant = match first {
            't' => Some(Consonant::T), 'k' => Some(Consonant::K), 'p' => Some(Consonant::P),
            'j' => Some(Consonant::J), 'w' => Some(Consonant::W), 'l' => Some(Consonant::L),
            's' => Some(Consonant::S), 'n' => Some(Consonant::N), 'm' => Some(Consonant::M),
            _ => None
        };
        if consonant.is_some() { first = chars.next().ok_or(SyllableParseError::NoVowel)?; }
        let vowel = match first {
            'a' => Vowel::A, 'e' => Vowel::E, 'i' => Vowel::I,
            'o' => Vowel::O, 'u' => Vowel::U,
            _ => return Err(if consonant.is_some() {
                    SyllableParseError::NoVowel
                } else {
                    SyllableParseError::EmptyString
                })
        };
        let mut nasal = false;
        if chars.as_str().starts_with("n") {
            let next = chars.as_str().chars().skip(1).next();
            if !next.is_some_and(|c| matches!(c, 'a'|'e'|'i'|'o'|'u')) {
                chars.next();
                nasal = true;
            }
        }
        *string = chars.as_str();
        return Ok(Syllable { consonant, vowel, nasal })
    }

    pub const fn encode(&self) -> u8 {
        let cons = if let Some(c) = self.consonant {c as u8} else {0};
        cons << 4 | (self.vowel as u8) << 1 | self.nasal as u8
    }
    pub const fn decode(byte: u8) -> Result<Self, SyllableDecodeError> {
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
        return Ok(Self { consonant, vowel, nasal })
    }
}

#[test]
fn test_parsing() {
    use {Consonant::*, Vowel::*};
    macro_rules! test_syl {
        ($lit: literal -> ($C: expr; $V: expr; $N: expr; $s: literal)) => {
            let mut s = $lit;
            let v = Syllable::parse(&mut s);
            assert_eq!(s, $s);
            assert_eq!(v, Ok(Syllable { consonant: $C, vowel: $V, nasal: $N }))
        };
        ($lit: literal -> (X, $idx: expr)) => {
            let v = Syllable::parse(&mut $lit);
            assert_eq!(v, Err($idx));
        };
    }
    test_syl!("jan Misali" -> (Some(J); A; true; " Misali"));
    test_syl!("jaaaa" -> (Some(J); A; false; "aaa"));
    test_syl!("ananana" -> (None; A; false; "nanana"));
    test_syl!("aaa" -> (None; A; false; "aa"));
    test_syl!("nja :3" -> (X, SyllableParseError::NoVowel));
    test_syl!("toki" -> (Some(T); O; false; "ki"));
    test_syl!("pona" -> (Some(P); O; false; "na"));
    test_syl!("nanpa" -> (Some(N); A; true; "pa"));
    test_syl!("" -> (X, SyllableParseError::EmptyString));
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