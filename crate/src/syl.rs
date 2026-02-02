use core::fmt::Display;

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
pub enum SoundSegmentKind {
    Vowel(Vowel),
    Consonant(Consonant),
    Space
}
impl Display for SoundSegmentKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Vowel(v) => write!(f, "{v}"),
            Self::Consonant(c) => write!(f, "{c}"),
            Self::Space => write!(f, "Space"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SoundSegment {
    pub kind: SoundSegmentKind,
    pub length: f32,
    pub frequency: f32
}

impl core::fmt::Display for SoundSegment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.kind == SoundSegmentKind::Space {
            write!(f, "SoundSegment::<Space> {{ len: {:.03} }}", self.length)
        } else {
            write!(f, "SoundSegment::<{}> {{ len: {:.03}, freq: {:.03} }}", self.kind, self.length, self.frequency)
        }
    }
}

impl SoundSegment {
    pub const fn vowel(vow: Vowel, length: f32, frequency: f32) -> Self {
        Self { kind: SoundSegmentKind::Vowel(vow), length, frequency }
    }
    pub const fn consonant(con: Consonant, length: f32, frequency: f32) -> Self {
        Self { kind: SoundSegmentKind::Consonant(con), length, frequency }
    }
    pub const fn space(length: f32) -> Self {
        Self { kind: SoundSegmentKind::Space, length, frequency: 0.0 }
    }
}