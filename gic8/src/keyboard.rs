//! Proper keyboard input for CHIP-8.
//!
//! <https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#keypad>
//!
//! | CHIP-8 | QWERTY |
//! |--------|--------|
//! |    1   |    1   |
//! |    2   |    2   |
//! |    3   |    3   |
//! |    C   |    4   |
//! |    4   |    Q   |
//! |    5   |    W   |
//! |    6   |    E   |
//! |    D   |    R   |
//! |    7   |    A   |
//! |    8   |    S   |
//! |    9   |    D   |
//! |    F   |    F   |
//! |    A   |    Z   |
//! |    0   |    X   |
//! |    B   |    C   |
//! |    F   |    V   |

use winit::{
    event::KeyEvent,
    keyboard::{PhysicalKey, KeyCode},
};

use std::sync::atomic::{AtomicBool, Ordering};

use crabchip8::interpreter::providers::KeyboardProvider;

/// Keeps track of which keys are pressed.
#[derive(Debug)]
pub struct KeyTracker {
    // Stored in the order of their hex digits on CHIP-8.
    pressed: [AtomicBool; 16],
}

impl KeyTracker {
    /// Creates a `KeyTracker` with no keys pressed.
    pub fn new() -> Self {
        KeyTracker { pressed: [const { AtomicBool::new(false) }; 16] }
    }

    pub fn process_event(&self, event: KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else { return };
        let index = match code {
            KeyCode::Digit1 => 0x1,
            KeyCode::Digit2 => 0x2,
            KeyCode::Digit3 => 0x3,
            KeyCode::Digit4 => 0xC,
            KeyCode::KeyQ   => 0x4,
            KeyCode::KeyW   => 0x5,
            KeyCode::KeyE   => 0x6,
            KeyCode::KeyR   => 0xD,
            KeyCode::KeyA   => 0x7,
            KeyCode::KeyS   => 0x8,
            KeyCode::KeyD   => 0x9,
            KeyCode::KeyF   => 0xE,
            KeyCode::KeyZ   => 0xA,
            KeyCode::KeyX   => 0x0,
            KeyCode::KeyC   => 0xB,
            KeyCode::KeyV   => 0xF,
            _ => return,
        };
        let is_pressed = event.state.is_pressed();
        self.pressed[index].store(is_pressed, Ordering::Release);
    }
}

impl KeyboardProvider for KeyTracker {
    fn is_key_pressed(&self, key: u8) -> bool {
        self.pressed[usize::from(key)].load(Ordering::Acquire)
    }
}
