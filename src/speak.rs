use std::f32;

use crate::syl::{Consonant, Syllable, Vowel};

#[derive(Debug, Copy, Clone, Default)]
struct WaveSettings {
    amplitude: f32,
    frequency: f32,
}
#[derive(Debug, Copy, Clone, Default)]
struct NoiseSettings {
    amplitude: f32,
    low_pass: f32,
    high_pass: f32,
    attack: bool,
}
#[derive(Debug, Copy, Clone, Default)]
struct SoundSettings {
    waves: [WaveSettings; 3],
    noise: NoiseSettings,
    is_consonant: bool,
    is_space: bool
}
impl SoundSettings {
    const fn new() -> Self {
        Self {
            waves: [WaveSettings {
                amplitude: 0.0,
                frequency: 0.0,
            }; 3],
            noise: NoiseSettings {
                amplitude: 0.0,
                low_pass: 1.0,
                high_pass: 0.0,
                attack: false,
            },
            is_consonant: false,
            is_space: false
        }
    }
    const fn with_waves(mut self, f1: f32, f2: f32, f3: f32, a1: f32, a2: f32, a3: f32) -> Self {
        self.waves = [
            WaveSettings {
                amplitude: a1,
                frequency: f1,
            },
            WaveSettings {
                amplitude: a2,
                frequency: f2,
            },
            WaveSettings {
                amplitude: a3,
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
        self.is_consonant = true;
        self
    }
}

static VOWEL_SETTINGS: [SoundSettings; 5] = [
    SoundSettings::new().with_waves(700.0, 1220.0, 2600.0, 0.80, 0.50, 1.00), // A
    SoundSettings::new().with_waves(530.0, 1680.0, 2500.0, 0.30, 0.45, 1.00), // E
    SoundSettings::new().with_waves(270.0, 2290.0, 3010.0, 1.00, 0.60, 0.50), // I
    SoundSettings::new().with_waves(570.0,  840.0, 2410.0, 1.00, 0.80, 0.30), // O
    SoundSettings::new().with_waves(300.0,  870.0, 2240.0, 1.00, 0.70, 0.20), // U
];

static CONSONANT_SETTINGS: [SoundSettings; 10] = [
    SoundSettings::new().consonant(), // None
    SoundSettings::new()
        .consonant()
        .with_noise(2.0, 1.0, 0.8)
        .attack(), // T
    SoundSettings::new()
        .consonant()
        .with_noise(1.8, 0.4, 0.2)
        .attack(), // K
    SoundSettings::new()
        .consonant()
        .with_noise(3.5, 0.04, 0.0)
        .attack(), // P
    SoundSettings::new()
        .consonant()
        .with_waves(380.0, 1150.0, 2900.0, 0.5, 0.5, 0.8), // L
    SoundSettings::new()
        .consonant()
        .with_waves(300.0, 610.0, 2200.0, 0.8, 0.4, 0.2), // W
    SoundSettings::new()
        .consonant()
        .with_noise(0.6, 1.0, 0.5), // S
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

trait SoundLike {
    fn get_settings(&self) -> SoundSettings;
}

impl SoundLike for Vowel {
    fn get_settings(&self) -> SoundSettings {
        VOWEL_SETTINGS[*self as u8 as usize]
    }
}
impl SoundLike for Consonant {
    fn get_settings(&self) -> SoundSettings {
        CONSONANT_SETTINGS[*self as u8 as usize]
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl WaveSettings {
    fn lerp(&self, other: Self, time: f32) -> Self {
        WaveSettings {
            amplitude: lerp(self.amplitude, other.amplitude, time),
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
            attack: if time > 0.05 {
                other.attack
            } else {
                self.attack
            },
        }
    }
}

impl SoundSettings {
    fn space() -> Self {
        Self {
            is_space: true,
            ..Self::default()
        }
    }

    fn lerp(&self, mut other: Self, time: f32) -> Self {
        if other.noise.attack && time < 0.05 {
            return SoundSettings::new();
        } else if other.noise.attack && time < 0.15 {
            let burst_t = (time - 0.05) / 0.10;
            let envelope = (1.0 - burst_t).powf(4.0);
            other.noise.amplitude = other.noise.amplitude * envelope * 3.0;
        }
        if time < 0.15 {
            return SoundSettings {
                waves: [
                    self.waves[0].lerp(other.waves[0], time / 0.15),
                    self.waves[1].lerp(other.waves[1], time / 0.15),
                    self.waves[2].lerp(other.waves[2], time / 0.15),
                ],
                noise: if other.noise.attack { other.noise } else { self.noise.lerp(other.noise, time / 0.15) },
                is_consonant: other.is_consonant,
                is_space: other.is_space
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
    phases: [f32; 3],
    last_low: f32,
    last_high: f32,
    rand_state: RandomState,
}

impl SynthState {
    fn synthesize(&mut self, sample_rate: u32) -> f32 {
        let settings = self.sound;

        let noise = self.rand_state.next_f32();
        self.last_low += settings.noise.low_pass * (noise - self.last_low);
        let low_passed = self.last_low;
        self.last_high += settings.noise.high_pass * (low_passed - self.last_high);
        let mut res = (low_passed - self.last_high) * settings.noise.amplitude;

        const TAU: f32 = 2.0 * core::f32::consts::PI;
        for i in 0..3 {
            let wave = settings.waves[i];
            self.phases[i] += (wave.frequency / (sample_rate as f32)) * TAU;
            if self.phases[i] > TAU {
                self.phases[i] -= TAU;
            }
            res += self.phases[i].sin() * wave.amplitude;
        }

        res
    }
}

pub struct SynthSettings {
    pub sample_rate: u32,
    pub consonant_time: f32,
    pub vowel_time: f32,
}

pub fn pronounce_syllables(
    syl: impl Iterator<Item = Option<Syllable>>,
    settings: SynthSettings,
) -> impl Iterator<Item = f32> {
    let SynthSettings {
        sample_rate,
        consonant_time,
        vowel_time,
    } = settings;
    let mut setting_iter = syl
        .map(|syl| [
            syl.and_then(|s| s.is_space.then(SoundSettings::space).or(s.consonant.map(|c| c.get_settings()))),
            Some(syl.map_or_else(SoundSettings::space, |s| s.is_space.then(SoundSettings::space).unwrap_or(s.vowel.get_settings()))),
            syl.and_then(|s| s.is_space.then(SoundSettings::space).or(if s.nasal { Some(Consonant::N.get_settings()) } else { None })),
        ]).flatten();
    let mut last_settings = SoundSettings::new();
    let mut target_settings = SoundSettings::new();
    let mut state = SynthState::default();
    let mut samp = 0;
    let mut start_time = 0f32;
    let mut current_duration = 0f32;
    core::iter::from_fn(move || {
        let abs_time = (samp as f32) / (sample_rate as f32);
        if start_time + current_duration <= abs_time {
            start_time = abs_time;
            last_settings = target_settings;
            if last_settings.is_space {
                last_settings = target_settings;
                last_settings.is_space = true;
                last_settings.is_consonant = true;
                last_settings.noise.amplitude = 0.0;
                last_settings.waves[0].amplitude = 0.0;
                last_settings.waves[1].amplitude = 0.0;
                last_settings.waves[2].amplitude = 0.0;
            }
            target_settings = loop {
                let Some(b) = setting_iter.next()? else { continue };
                break b;
            };
            if target_settings.is_space {
                target_settings = last_settings;
                target_settings.is_space = true;
                target_settings.is_consonant = true;
                target_settings.noise.amplitude = 0.0;
                target_settings.waves[0].amplitude = 0.0;
                target_settings.waves[1].amplitude = 0.0;
                target_settings.waves[2].amplitude = 0.0;
            }
            eprintln!("Switching to target at {abs_time}: {target_settings:?}");
            current_duration = if target_settings.is_consonant { consonant_time } else { vowel_time };
        }
        let factor = (abs_time - start_time) / current_duration;
        state.sound = last_settings.lerp(target_settings, factor);
        let res = state.synthesize(sample_rate);
        samp += 1;
        Some(res)
    })
}
