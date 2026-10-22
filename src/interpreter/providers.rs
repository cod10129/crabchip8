//! Providers of various resources to the `Chip8Interpreter`.

use std::{hash, time::Instant};

#[cfg(doc)]
use super::Chip8Interpreter;

/// A type that can provide randomness to the [`Chip8Interpreter`].
pub trait RandomnessProvider {
    /// Generates a random number from 0 to 255.
    fn random(&mut self) -> u8;
}

/// A type that can provide keyboard inputs to the [`Chip8Interpreter`].
pub trait KeyboardProvider {
    /// Is the key specified by `key` currently pressed?
    ///
    /// `key` must be less than 16/`0x10`.
    fn is_key_pressed(&mut self, key: u8) -> bool;

    /// Returns the corresponding digit `0-F` if a key is currently pressed,
    /// and `0xFF` otherwise.
    fn is_any_key_pressed(&mut self) -> u8 {
        for n in 0..16 {
            if self.is_key_pressed(n) {
                return n;
            }
        }
        0xFF
    }
}

/// A type that can provide sound to the [`Chip8Interpreter`].
pub trait SoundProvider {
    /// Enables sound output.
    fn enable_sound(&mut self);
    /// Disables sound output.
    fn disable_sound(&mut self);
}

/// A type that can provide time to the [`Chip8Interpreter`].
///
/// It processes "ticks" that are at a rate of 60 Hz.
pub trait TimeProvider {
    /// Returns the number of ticks that have passed since some arbitrary epoch.
    fn current_tick_count(&mut self) -> u64;
}

/// A [`RandomnessProvider`] that uses [`std::hash::RandomState`] as a hacky
/// source for randomness.
#[derive(Debug, Default)]
pub struct HashRandomnessProvider {
    state: hash::RandomState,
    hash: u64,
}

impl HashRandomnessProvider {
    /// Creates a new `HashRandomnessProvider` with a random state.
    pub fn new() -> Self {
        HashRandomnessProvider {
            state: hash::RandomState::new(),
            hash: 0xF2AA_B555,
        }
    }
}

impl RandomnessProvider for HashRandomnessProvider {
    #[allow(clippy::cast_possible_truncation)]
    fn random(&mut self) -> u8 {
        self.hash = hash::BuildHasher::hash_one(&self.state, self.hash);
        self.hash as u8
    }
}

/// A dummy [`KeyboardProvider`] where no keys are ever pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct DummyKeyboardProvider;

impl KeyboardProvider for DummyKeyboardProvider {
    fn is_key_pressed(&mut self, _: u8) -> bool {
        false
    }

    fn is_any_key_pressed(&mut self) -> u8 {
        0xFF
    }
}

/// A [`SoundProvider`] that does nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct DummySoundProvider {
    /// Whether or not sound is supposed to be enabled.
    pub sound_enabled: bool,
}

impl DummySoundProvider {
    /// Creates a new `DummySoundProvider` that is not playing sound.
    pub const fn new() -> Self {
        DummySoundProvider { sound_enabled: false }
    }
}

impl SoundProvider for DummySoundProvider {
    fn enable_sound(&mut self) { self.sound_enabled = true }
    fn disable_sound(&mut self) { self.sound_enabled = false }
}

/// A [`TimeProvider`] that uses [`std::time::Instant`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StdTimeProvider {
    epoch: Instant,
}

impl StdTimeProvider {
    /// Creates a new `StdTimeProvider` with its epoch set to the current moment.
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        StdTimeProvider { epoch: Instant::now() }
    }
}

impl TimeProvider for StdTimeProvider {
    fn current_tick_count(&mut self) -> u64 {
        const NANOS_PER_TICK: u64 = 1_000_000_000 / 60;
        let dur = Instant::now().duration_since(self.epoch);
        (dur.as_secs() * 60) + (u64::from(dur.subsec_nanos()) / NANOS_PER_TICK)
    }
}
