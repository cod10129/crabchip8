//! A CHIP-8 emulator.

use std::{fmt, ops::{Index, IndexMut}};

/// The CHIP-8 memory.
///
/// 4096 bytes, with the range `0x200-0xE8F` formally available to programs.
#[derive(Clone, PartialEq, Eq)]
pub struct Memory([u8; 4096]);

impl Memory {
    /// Constructs a new `Memory`, including preloaded font data.
    pub fn new() -> Memory {
        // https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference#fonts
        const FONT_DATA: [u8; 0x50] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0,
            0x20, 0x60, 0x20, 0x20, 0x70,
            0xF0, 0x10, 0xF0, 0x80, 0xF0,
            0xF0, 0x10, 0xF0, 0x10, 0xF0,
            0x90, 0x90, 0xF0, 0x10, 0x10,
            0xF0, 0x80, 0xF0, 0x10, 0xF0,
            0xF0, 0x80, 0xF0, 0x90, 0xF0,
            0xF0, 0x10, 0x20, 0x40, 0x40,
            0xF0, 0x90, 0xF0, 0x90, 0xF0,
            0xF0, 0x90, 0xF0, 0x10, 0xF0,
            0xF0, 0x90, 0xF0, 0x90, 0x90,
            0xE0, 0x90, 0xE0, 0x90, 0xE0,
            0xF0, 0x80, 0x80, 0x80, 0xF0,
            0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0,
            0xF0, 0x90, 0xF0, 0x80, 0x80,
        ];
        let mut arr = [0; 4096];
        let font_segment = &mut arr[0x100..0x150];
        font_segment.copy_from_slice(&FONT_DATA);

        Memory(arr)
    }
}

impl Default for Memory {
    fn default() -> Self {
        Memory::new()
    }
}

/// A label for a register, 0-F.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegLabel(u8);

impl RegLabel {
    /// Creates a new `RegLabel` from an integer in the range [0, 15].
    ///
    /// # Panics
    /// Panics if `n > 15`.
    pub fn new(n: u8) -> Self {
        assert!(n < 16, "RegLabel value {n:#X} was greater than 0x0F");
        RegLabel(n)
    }
}

impl fmt::Debug for RegLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V{:X}", self.0)
    }
}

/// One of the CHIP-8 [`Registers`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Register(pub u8);

/// The registers of the CHIP-8 system, `V0` through `VF`.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Registers([Register; 16]);

impl Registers {
    /// Creates a set of `Registers` with all registers initialized to `0x00`.
    pub const fn new() -> Registers {
        Registers([const { Register(0) }; 16])
    }
}

impl Index<RegLabel> for Registers {
    type Output = Register;

    fn index(&self, index: RegLabel) -> &Register {
        &self.0[usize::from(index.0)]
    }
}

impl IndexMut<RegLabel> for Registers {
    fn index_mut(&mut self, index: RegLabel) -> &mut Register {
        &mut self.0[usize::from(index.0)]
    }
}

/// The 12-bit address register `I`.
#[derive(Clone, PartialEq, Eq)]
pub struct IRegister(u16);

impl IRegister {
    /// Instantiates the I register from an integer in the range [0, 0xFFF].
    ///
    /// # Panics
    /// Panics if `n > 0xFFF`.
    pub fn new(n: u16) -> Self {
        assert!(n < 0x1000, "IRegister value {n:#X} was greater than 0xFFF");
        IRegister(n)
    }
}

/// The graphics state of the CHIP-8 screen (64x32 grid of bits).
#[derive(Clone, PartialEq, Eq)]
// Row-major order.
pub struct GraphicsState([[bool; 64]; 32]);

impl GraphicsState {
    /// Creates a blank `GraphicsState` with all pixels set to 0.
    pub const fn new() -> GraphicsState {
        GraphicsState([[false; 64]; 32])
    }
}

impl Default for GraphicsState {
    fn default() -> Self {
        GraphicsState::new()
    }
}
