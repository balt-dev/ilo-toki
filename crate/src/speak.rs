use core::f32;

use crate::syl::{SoundSegment, SoundSegmentKind};

const MAX_HARMONICS: usize = 5;

trait FloatExt {
    const TAU: f32;
    fn saw(self) -> f32;
    fn sin(self) -> f32;
}
impl FloatExt for f32 {
    const TAU: f32 = 2.0 * f32::consts::PI;
    fn saw(mut self) -> f32 {
        self %= f32::TAU;
        1.0 - (self / f32::TAU)
    }
    fn sin(self) -> f32 {
        use f32::consts::*;
        const FRAC_3_PI_2: f32 = 3.0 * FRAC_PI_2;
        fn partial_sin(x: f32) -> f32 {
            let x2 = x * x;
            let x3 = x * x2;
            let x5 = x3 * x2;
            x - (x3 / 6.0) + (x5 / 120.0)
        }
        let mut x = self % TAU;
        if x < 0.0 { x += TAU; }
        if x < FRAC_PI_2 { partial_sin(x) }
        else if x < PI { partial_sin(PI - x) }
        else if x < FRAC_3_PI_2 { -partial_sin(x - PI) }
        else { -partial_sin(TAU - x) }
    }
}
const fn floor(f: f32) -> f32 {
    f - (f % 1.0)
}

#[derive(Debug, Copy, Clone, Default)]
struct WaveSettings {
    bandwidth: f32,
    frequency: f32,
}
#[derive(Debug, Copy, Clone, Default)]
struct NoiseSettings {
    amplitude: f32,
    low_pass: f32,
    high_pass: f32,
    attack: bool,
    dampening: f32
}
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
enum SoundKind {
    #[default]
    Vowel,
    Consonant,
    Space
}

#[derive(Debug, Copy, Clone, Default)]
struct SoundSettings {
    waves: [WaveSettings; 3],
    noise: NoiseSettings,
    kind: SoundKind,
    pitch: f32,
    duration: f32
}
impl SoundSettings {
    const fn new() -> Self {
        Self {
            waves: [WaveSettings {
                bandwidth: 0.0,
                frequency: 0.0,
            }; 3],
            noise: NoiseSettings {
                amplitude: 0.0,
                low_pass: 1.0,
                high_pass: 0.0,
                attack: false,
                dampening: 0.0
            },
            kind: SoundKind::Vowel,
            pitch: 0.0,
            duration: 0.0
        }
    }
    const fn with_waves(mut self, f1: f32, f2: f32, f3: f32, a1: f32, a2: f32, a3: f32) -> Self {
        self.waves = [
            WaveSettings {
                bandwidth: a1,
                frequency: f1,
            },
            WaveSettings {
                bandwidth: a2,
                frequency: f2,
            },
            WaveSettings {
                bandwidth: a3,
                frequency: f3,
            },
        ];
        self
    }
    const fn with_noise(mut self, amp: f32, low: f32, high: f32) -> Self {
        self.noise = NoiseSettings {
            amplitude: amp,
            low_pass: low,
            high_pass: high,
            ..self.noise
        };
        self
    }
    const fn attack(mut self) -> Self {
        self.noise.attack = true;
        self
    }
    const fn consonant(mut self) -> Self {
        self.kind = SoundKind::Consonant;
        self
    }
    const fn dampening(mut self, factor: f32) -> Self {
        self.noise.dampening = factor;
        self
    }
    const fn normalize_to(mut self, target: f32) -> Self {
        const fn snap(f: &mut f32, target: f32) {
            *f = floor(*f / target) * target;
        }

        snap(&mut self.waves[0].frequency, target);
        snap(&mut self.waves[1].frequency, target);
        snap(&mut self.waves[2].frequency, target);
        
        self
    }
}

static VOWEL_SETTINGS: [SoundSettings; 5] = [
    SoundSettings::new().with_waves(700.0, 1220.0, 2600.0, 0.80, 0.50, 1.00).normalize_to(220.0), // A
    SoundSettings::new().with_waves(530.0, 1680.0, 2500.0, 0.30, 0.45, 1.00).normalize_to(220.0), // E
    SoundSettings::new().with_waves(270.0, 2290.0, 3010.0, 1.00, 0.60, 0.50).normalize_to(220.0), // I
    SoundSettings::new().with_waves(570.0,  840.0, 2410.0, 1.00, 0.80, 0.30).normalize_to(220.0), // O
    SoundSettings::new().with_waves(300.0,  870.0, 2240.0, 1.00, 0.70, 0.20).normalize_to(220.0), // U
];

static CONSONANT_SETTINGS: [SoundSettings; 9] = [
    SoundSettings::new()
        .consonant()
        .with_noise(2.0, 1.0, 0.8)
        .attack()
        .dampening(0.1), // T
    SoundSettings::new()
        .consonant()
        .with_noise(1.8, 0.4, 0.2)
        .attack()
        .dampening(0.5), // K
    SoundSettings::new()
        .consonant()
        .with_noise(3.5, 0.04, 0.0)
        .dampening(0.8)
        .attack(), // P
    SoundSettings::new()
        .consonant()
        .with_waves(380.0, 1150.0, 2900.0, 0.5, 0.5, 0.8), // L
    SoundSettings::new()
        .consonant()
        .with_waves(300.0, 610.0, 2200.0, 0.8, 0.4, 0.2), // W
    SoundSettings::new()
        .consonant()
        .with_waves(5000.0, 0.0, 0.0, 0.1, 0.0, 0.0)
        .with_noise(0.6, 1.0, 0.5)
        .dampening(0.05), // S
    SoundSettings::new()
        .consonant()
        .with_waves(280.0, 2200.0, 2800.0, 0.4, 0.9, 0.9), // J
    SoundSettings::new()
        .consonant()
        .with_waves(250.0, 1100.0, 2500.0, 0.8, 0.2, 0.1), // N
    SoundSettings::new()
        .consonant()
        .with_waves(250.0, 800.0, 2300.0, 0.9, 0.05, 0.03), // M
];

impl SoundSegment {
    fn get_settings(&self) -> SoundSettings {
        let mut seg = match self.kind {
            SoundSegmentKind::Space => SoundSettings::space(),
            SoundSegmentKind::Vowel(vow) => VOWEL_SETTINGS[vow as u8 as usize],
            SoundSegmentKind::Consonant(con) => CONSONANT_SETTINGS[con as u8 as usize]
        };
        seg.pitch = self.frequency;
        seg.duration = self.length;
        seg
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl WaveSettings {
    fn lerp(&self, other: Self, time: f32) -> Self {
        WaveSettings {
            bandwidth: lerp(self.bandwidth, other.bandwidth, time),
            frequency: lerp(self.frequency, other.frequency, time),
        }
    }
}

impl NoiseSettings {
    fn lerp(&self, other: Self, time: f32) -> Self {
        NoiseSettings {
            amplitude: lerp(self.amplitude, other.amplitude, time),
            low_pass: lerp(self.low_pass, other.low_pass, time),
            high_pass: lerp(self.high_pass, other.high_pass, time),
            dampening: lerp(self.dampening, other.dampening, time),
            attack: other.attack,
        }
    }
}

impl SoundSettings {
    fn space() -> Self {
        Self {
            kind: SoundKind::Space,
            ..Self::default()
        }
    }

    fn lerp(&self, mut other: Self, time: f32) -> Self {
        if other.noise.attack && time < 0.05 {
            let mut ret = SoundSettings::new();
            ret.duration = other.duration;
            return ret;
        } else if other.noise.attack {
            let burst_t = (time - 0.05) / 0.95;
            let envelope = 1.0 - burst_t;
            other.noise.amplitude = other.noise.amplitude * envelope * envelope * 3.0;
        }
        if time < 0.15 {
            return SoundSettings {
                waves: [
                    self.waves[0].lerp(other.waves[0], time / 0.15),
                    self.waves[1].lerp(other.waves[1], time / 0.15),
                    self.waves[2].lerp(other.waves[2], time / 0.15),
                ],
                noise: if other.noise.attack { other.noise } else { self.noise.lerp(other.noise, time / 0.15) },
                kind: other.kind,
                pitch: lerp(self.pitch, other.pitch, time / 0.15),
                duration: other.duration
            };
        } else { return other }
    }
}

#[derive(Debug, Copy, Clone)]
struct RandomState([u32; 4]);

impl RandomState {
    const fn next(&mut self) -> u32 {
        let s = &mut self.0;
        let res = s[0].wrapping_add(s[3]).rotate_left(7).wrapping_add(s[0]);
        let t = s[1] << 9;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(11);
        res
    }
    /// Generates a random float on [-1, 1)
    const fn next_f32(&mut self) -> f32 {
        const BITS: u32 = 0b0_10000000_00000000000000000000000;
        const MASK: u32 = 0b0_00000000_11111111111111111111111;
        f32::from_bits((self.next() & MASK) | BITS) - 3.0
    }
    const fn fixed() -> Self {
        Self([0xFEEDBACC, 0xDEADBEEF, 0xE6E6E6E6, 0xC8675309])
    }
}

impl Default for RandomState {
    fn default() -> Self {
        Self::fixed()
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct SynthState {
    sound: SoundSettings,
    last_low: f32,
    last_high: f32,
    rand_state: RandomState,
    brown_state: f32,
    phases: [f32; MAX_HARMONICS]
}

impl SynthState {
    fn synthesize(&mut self, sample_rate: u32) -> f32 {
        let settings = self.sound;

        let white = self.rand_state.next_f32();
        self.brown_state = (self.brown_state * (self.sound.noise.dampening)) + (white * (1.0 - self.sound.noise.dampening));
        let noise = self.brown_state;
        self.last_low += settings.noise.low_pass * (noise - self.last_low);
        let low_passed = self.last_low;
        self.last_high += settings.noise.high_pass * (low_passed - self.last_high);
        let mut res = (low_passed - self.last_high) * settings.noise.amplitude;

        for i in 0..3 {
            let wave = settings.waves[i];
            self.phases[i] += (wave.frequency * (settings.pitch / 440.0) / (sample_rate as f32)) * f32::TAU;
            if self.phases[i] > f32::TAU {
                self.phases[i] -= f32::TAU;
            }
            let amp = if i == 1 { self.phases[i].saw() } else { self.phases[i].sin() };
            res += amp * wave.bandwidth;
        }

        res
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SynthSettings {
    pub sample_rate: u32,
    pub consonant_time: f32,
    pub vowel_time: f32,
    pub space_time: f32,
    pub base_frequency: f32
}

impl Default for SynthSettings {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            consonant_time: 0.03,
            vowel_time: 0.3,
            space_time: 0.08,
            base_frequency: 440.0
        }
    }
}

pub fn pronounce_segments(
    mut syl: impl Iterator<Item = SoundSegment>,
    settings: SynthSettings,
) -> impl Iterator<Item = f32> {
    let SynthSettings {
        sample_rate,
        base_frequency,
        ..
    } = settings;
    let mut last_settings = SoundSettings::new();
    last_settings.pitch = base_frequency;
    let mut target_settings = SoundSettings::new();
    let mut state = SynthState::default();
    let mut samp = 0;
    let mut start_time = 0f32;
    let mut current_duration = 0f32;
    let mut done = false;
    core::iter::from_fn(move || {
        if done { return None; }
        let abs_time = (samp as f32) / (sample_rate as f32);
        if start_time + current_duration <= abs_time {
            start_time = abs_time;
            last_settings = target_settings;
            if last_settings.kind == SoundKind::Space {
                last_settings = target_settings;
                last_settings.kind = SoundKind::Space;
                last_settings.noise.amplitude = 0.0;
                last_settings.waves[0].bandwidth = 0.0;
                last_settings.waves[1].bandwidth = 0.0;
                last_settings.waves[2].bandwidth = 0.0;
            }
            let Some(t) = syl.next().map(|seg| seg.get_settings()) else { done = true; return None; };
            target_settings = t;
            if target_settings.kind == SoundKind::Space {
                let len = target_settings.duration;
                target_settings = last_settings;
                target_settings.duration = len;
                target_settings.kind = SoundKind::Space;
                target_settings.noise.amplitude = 0.0;
                target_settings.waves[0].bandwidth = 0.0;
                target_settings.waves[1].bandwidth = 0.0;
                target_settings.waves[2].bandwidth = 0.0;
            }
            current_duration = target_settings.duration;
        }
        let factor = (abs_time - start_time) / current_duration;
        state.sound = last_settings.lerp(target_settings, factor);
        let res = state.synthesize(sample_rate);
        samp += 1;
        Some(res)
    })
}

pub fn pronounce_single(freqs: [f32; 3], bands: [f32; 3], pitch: f32, time: f32, sample_rate: u32) -> impl Iterator<Item = f32> {
    let samples = (time * (sample_rate as f32)) as usize;
    let mut state = SynthState {
        sound: SoundSettings { waves: [
            WaveSettings { frequency: freqs[0], bandwidth: bands[0] },
            WaveSettings { frequency: freqs[1], bandwidth: bands[1] },
            WaveSettings { frequency: freqs[2], bandwidth: bands[2] },
        ], pitch, ..Default::default() }.normalize_to(220.0),
        ..Default::default()
    };
    (0..samples).map(move |_| {
        state.synthesize(sample_rate)
    })
}