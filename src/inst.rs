//! Instructions for CHIP-8.
#![allow(missing_docs)]

use crate::{Address, RegLabel};

pub enum Instruction {
    ClearScreen, // `00E0`
    ReturnFromSubroutine, // `00EE`
    Jump { nnn: Address }, // `1NNN`
    ExecuteSubroutine { nnn: Address }, // `2NNN`
    SkipIfEqual { x: RegLabel, nn: u8 }, // `3XNN`
    SkipIfNotEqual { x: RegLabel, nn: u8 }, // `4XNN`
    SkipIfRegistersEqual { x: RegLabel, y: RegLabel }, // `5XY0`
    StoreInRegister { x: RegLabel, nn: u8 }, // `6XNN`
    AddToRegister { x: RegLabel, nn: u8 }, // `7XNN`
    StoreRegister { x: RegLabel, y: RegLabel }, // `8XY0`
    RegisterOr { x: RegLabel, y: RegLabel }, // `8XY1`
    RegisterAnd { x: RegLabel, y: RegLabel }, // `8XY2`
    RegisterXor { x: RegLabel, y: RegLabel }, // `8XY3`
    AddRegisters { x: RegLabel, y: RegLabel }, // `8XY4`
    SubtractRegisters { x: RegLabel, y: RegLabel }, // `8XY5`
    RegisterShiftRight { x: RegLabel, y: RegLabel }, // `8XY6`
    SubtractFromRegister { x: RegLabel, y: RegLabel }, // `8XY7`
    RegisterShiftLeft { x: RegLabel, y: RegLabel }, // `8XYE`
    SkipIfRegistersNotEqual { x: RegLabel, y: RegLabel }, // `9XY0`
    StoreAddress { nnn: Address }, // `ANNN`
    JumpWithOffset { nnn: Address }, // `BNNN`
    RandomWithMask { x: RegLabel, nn: u8 }, // `CXNN`
    DrawSprite { x: RegLabel, y: RegLabel, n: u8 }, // `DXYN`
    SkipIfKeyPressed { x: RegLabel }, // `EX9E`
    SkipIfKeyNotPressed { x: RegLabel }, // `EXA1`
    LoadDelayTimer { x: RegLabel }, // `FX07`
    WaitForKeypress { x: RegLabel }, // `FX0A`
    SetDelayTimer { x: RegLabel }, // `FX15`
    SetSoundTimer { x: RegLabel }, // `FX18`
    AddToIRegister { x: RegLabel }, // `FX1E`
    SetToPreloadedSpriteData { x: RegLabel }, // `FX29`
    StoreBCDToMemory { x: RegLabel }, // `FX33`
    StoreToMemory { x: RegLabel }, // `FX55`
    LoadFromMemory { x: RegLabel }, // `FX65`
}

impl Instruction {
    /// Parses an `Instruction` from the high and low bytes.
    pub fn parse(hi: u8, lo: u8) -> Result<Instruction, (u8, u8)> {
        let hi_hi = hi >> 4;
        let hi_lo = hi & 0b1111;
        let lo_hi = lo >> 4;
        let lo_lo = lo & 0b1111;

        let nnn = Address::new(u16::from_be_bytes([hi, lo]) & 0x3F);
        let nn = lo;
        let x = RegLabel::new(hi_lo);
        let y = RegLabel::new(lo_hi);

        macro_rules! failed {
            () => { return Err((hi, lo)) };
        }

        Ok(match hi_hi {
            // Address instructions
            0x1 => Instruction::Jump { nnn },
            0x2 => Instruction::ExecuteSubroutine { nnn },
            0xA => Instruction::StoreAddress { nnn },
            0xB => Instruction::JumpWithOffset { nnn },
            // Register + value instructions
            0x3 => Instruction::SkipIfEqual { x, nn },
            0x4 => Instruction::SkipIfNotEqual { x, nn },
            0x6 => Instruction::StoreInRegister { x, nn },
            0x7 => Instruction::AddToRegister { x, nn },
            0xC => Instruction::RandomWithMask { x, nn },
            // Argumentless instructions
            0x0 => match lo {
                0xE0 => Instruction::ClearScreen,
                0xEE => Instruction::ReturnFromSubroutine,
                _ => failed!(),
            },
            // Single register instructions
            0xE => match lo {
                0x9E => Instruction::SkipIfKeyPressed { x },
                0xA1 => Instruction::SkipIfKeyNotPressed { x },
                _ => failed!(),
            },
            0xF => match lo {
                0x07 => Instruction::LoadDelayTimer { x },
                0x0A => Instruction::WaitForKeypress { x },
                0x15 => Instruction::SetDelayTimer { x },
                0x18 => Instruction::SetSoundTimer { x },
                0x1E => Instruction::AddToIRegister { x },
                0x29 => Instruction::SetToPreloadedSpriteData { x },
                0x33 => Instruction::StoreBCDToMemory { x },
                0x55 => Instruction::StoreToMemory { x },
                0x65 => Instruction::LoadFromMemory { x },
                _ => failed!(),
            },
            // Double register instructions
            0x5 if lo_lo == 0 => Instruction::SkipIfRegistersEqual { x, y },
            0x9 if lo_lo == 0 => Instruction::SkipIfRegistersNotEqual { x, y },
            0x8 => match lo_lo {
                0x0 => Instruction::StoreRegister { x, y },
                0x1 => Instruction::RegisterOr { x, y },
                0x2 => Instruction::RegisterAnd { x, y },
                0x3 => Instruction::RegisterXor { x, y },
                0x4 => Instruction::AddRegisters { x, y },
                0x5 => Instruction::SubtractRegisters { x, y },
                0x6 => Instruction::RegisterShiftRight { x, y },
                0x7 => Instruction::SubtractFromRegister { x, y },
                0xE => Instruction::RegisterShiftLeft { x, y },
                0x10.. => unreachable!(),
                _ => failed!(),
            },
            0xD => Instruction::DrawSprite { x, y, n: lo_lo },
            0x10.. => unreachable!(),
            0x5 | 0x9 => failed!(),
        })

    }
}
