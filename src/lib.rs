//! A CHIP-8 emulator.

use std::{fmt, ops::{Add, AddAssign, Index, IndexMut, Sub, SubAssign}};

pub mod inst;
pub mod interpreter;

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

const FONT_DATA_BASE_ADDRESS: u16 = 0x100;

/// The CHIP-8 memory.
///
/// 4096 bytes, with the range `0x200-0xE8F` formally available to programs.
#[derive(Clone, PartialEq, Eq)]
pub struct Memory([u8; 4096]);

impl Memory {
    /// Constructs a new `Memory`, including preloaded font data.
    pub fn new() -> Memory {
        let mut arr = [0; 4096];
        let font_segment = &mut arr[usize::from(FONT_DATA_BASE_ADDRESS)..][..0x50];
        font_segment.copy_from_slice(&FONT_DATA);

        Memory(arr)
    }
}

impl Default for Memory {
    fn default() -> Self {
        Memory::new()
    }
}

impl Index<Address> for Memory {
    type Output = u8;

    fn index(&self, index: Address) -> &u8 {
        &self.0[usize::from(index.0)]
    }
}

impl IndexMut<Address> for Memory {
    fn index_mut(&mut self, index: Address) -> &mut u8 {
        &mut self.0[usize::from(index.0)]
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

    /// Returns the numerical value of this `RegLabel`. The value is guaranteed
    /// to be less than 16.
    pub fn as_number(self) -> u8 {
        self.0
    }
}

#[allow(missing_docs)]
impl RegLabel {
    pub const V0: RegLabel = RegLabel(0x0);
    pub const V1: RegLabel = RegLabel(0x1);
    pub const V2: RegLabel = RegLabel(0x2);
    pub const V3: RegLabel = RegLabel(0x3);
    pub const V4: RegLabel = RegLabel(0x4);
    pub const V5: RegLabel = RegLabel(0x5);
    pub const V6: RegLabel = RegLabel(0x6);
    pub const V7: RegLabel = RegLabel(0x7);
    pub const V8: RegLabel = RegLabel(0x8);
    pub const V9: RegLabel = RegLabel(0x9);
    pub const VA: RegLabel = RegLabel(0xA);
    pub const VB: RegLabel = RegLabel(0xB);
    pub const VC: RegLabel = RegLabel(0xC);
    pub const VD: RegLabel = RegLabel(0xD);
    pub const VE: RegLabel = RegLabel(0xE);
    pub const VF: RegLabel = RegLabel(0xF);
}

impl fmt::Debug for RegLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V{:X}", self.0)
    }
}

/// One of the CHIP-8 [`Registers`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
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

/// A 12-bit address.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Address(u16);

impl Address {
    /// Creates a new `Address` from an integer in the range [0, 0xFFF].
    ///
    /// # Panics
    /// Panics if `n > 0xFFF`.
    pub fn new(n: u16) -> Self {
        assert!(n < 0x1000, "Address value {n:#X} was greater than 0xFFF");
        Address(n)
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#05X}", self.0)
    }
}

impl Add<u8> for Address {
    type Output = Address;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: u8) -> Self {
        // Bitwise AND here to wrap from 0xFFF to 0x000
        Self(self.0.wrapping_add(u16::from(rhs)) & 0xFFF)
    }
}

impl AddAssign<u8> for Address {
    fn add_assign(&mut self, rhs: u8) {
        *self = *self + rhs;
    }
}

/// Subtracts a `u8` from an `Address`, wrapping from `0x000` to `0xFFF`.
impl Sub<u8> for Address {
    type Output = Address;

    fn sub(self, rhs: u8) -> Self {
        let (mut difference, overflow) = self.0.overflowing_sub(u16::from(rhs));
        if overflow {
            difference -= 0xF000;
        }
        Self(difference)
    }
}

impl SubAssign<u8> for Address {
    fn sub_assign(&mut self, rhs: u8) {
        *self = *self - rhs;
    }
}

/// The graphics state of the CHIP-8 screen (64x32 grid of bits).
#[derive(Debug, Clone, PartialEq, Eq)]
// Row-major order.
pub struct GraphicsState([[u8; 8]; 32]);

impl GraphicsState {
    /// Creates a blank `GraphicsState` with all pixels set to 0.
    pub const fn blank() -> GraphicsState {
        GraphicsState([[0; 8]; 32])
    }

    /// Returns the graphical data.
    pub fn data(&self) -> [[u8; 8]; 32] {
        self.0
    }

    /// Returns an iterator over the pixels of each row.
    #[expect(clippy::missing_panics_doc, reason = "can't actually panic")]
    pub fn pixel_iterator(&self) -> impl Iterator<Item = [bool; 64]> {
        self.0.into_iter().map(|row| {
            let v = row.map(|byte|
                [
                    bool::try_from((byte >> 7) & 0b1).unwrap(),
                    bool::try_from((byte >> 6) & 0b1).unwrap(),
                    bool::try_from((byte >> 5) & 0b1).unwrap(),
                    bool::try_from((byte >> 4) & 0b1).unwrap(),
                    bool::try_from((byte >> 3) & 0b1).unwrap(),
                    bool::try_from((byte >> 2) & 0b1).unwrap(),
                    bool::try_from((byte >> 1) & 0b1).unwrap(),
                    bool::try_from( byte       & 0b1).unwrap(),
                ])
                .concat();
            <[bool; 64]>::try_from(v).unwrap()
        })
    }
}

impl Default for GraphicsState {
    fn default() -> Self {
        GraphicsState::blank()
    }
}

#[cfg(test)]
mod tests {
    use super::Address;

    #[test]
    fn address_addition() {
        let addr = Address::new(0xFF1);
        assert_eq!(addr + 0x0D, Address::new(0xFFE));
        assert_eq!(addr + 0x16, Address::new(0x007));
    }

    #[test]
    fn address_subtraction() {
        let addr = Address::new(0x010);
        assert_eq!(addr - 0x0A, Address::new(0x006));
        assert_eq!(addr - 0x11, Address::new(0xFFF));
    }
}
