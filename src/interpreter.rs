//! The CHIP-8 interpreter.

use crate::{Address, GraphicsState, Memory, RegLabel, Registers, inst::Instruction};

/// The container for all the information stored in the CHIP-8 interpreter.
pub struct Chip8Interpreter {
    memory: Memory,
    regs: Registers,
    i_register: Address,
    graphics: GraphicsState,
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<Address>,
    /// Program Counter
    pc: Address,
    /// Are we currently waiting for a keypress? (Instruction FX0A)
    waiting: bool,
    randomness_provider: Box<dyn RandomnessProvider>,
    keyboard_provider: Box<dyn KeyboardProvider>,
    // time_provider: Box<dyn TimeProvider>,
}

impl Chip8Interpreter {
    /// Initializes a new interpreter with no loaded program.
    pub fn new<R, K>(randomness_provider: R, keyboard_provider: K) -> Self
    where
        R: RandomnessProvider + 'static,
        K: KeyboardProvider + 'static
    {
        Chip8Interpreter {
            memory: Memory::new(),
            regs: Registers::new(),
            i_register: Address::new(0),
            graphics: GraphicsState::blank(),
            delay_timer: 0,
            sound_timer: 0,
            stack: Vec::with_capacity(16),
            pc: Address::new(0x200),
            waiting: false,
            randomness_provider: Box::new(randomness_provider),
            keyboard_provider: Box::new(keyboard_provider),
        }
    }

    /// Loads a CHIP-8 program into the interpreter.
    ///
    /// The program will be loaded starting at address `0x200`, and can be at
    /// most `0xE00` (3,584) bytes long.
    ///
    /// # Panics
    /// Panics if the program is longer than `0xE00` bytes.
    pub fn load_program(&mut self, program: &[u8]) {
        let len = program.len();
        assert!(len <= 0xE00, "Program length of {len:#X} bytes is larger than 0xE00");

        let program_space = &mut self.memory.0[0x200..][..len];
        program_space.copy_from_slice(program);
    }

    /// Moves the program counter forward by one instruction.
    fn increment_pc(&mut self) {
        self.pc += 2;
    }

    /// Executes one instruction on the interpreter, including moving the
    /// program counter.
    ///
    /// # Panics
    /// - Panics if the stack underflows while returning from a subroutine
    /// - Panics if a TODO instruction is executed
    #[expect(clippy::too_many_lines)]
    fn exec_instruction(&mut self, inst: Instruction) {
        self.increment_pc();
        match inst {
            Instruction::ClearScreen => { self.graphics = GraphicsState::blank() },
            Instruction::ReturnFromSubroutine => {
                self.pc = self.stack.pop().expect("stack underflow");
            },
            Instruction::Jump { nnn } => { self.pc = nnn },
            Instruction::ExecuteSubroutine { nnn } => {
                self.stack.push(self.pc);
                self.pc = nnn;
            },
            Instruction::SkipIfEqual { x, nn } => {
                if self.regs[x].0 == nn {
                    self.increment_pc();
                }
            },
            Instruction::SkipIfNotEqual { x, nn } => {
                if self.regs[x].0 != nn {
                    self.increment_pc();
                }
            },
            Instruction::SkipIfRegistersEqual { x, y } => {
                if self.regs[x] == self.regs[y] {
                    self.increment_pc();
                }
            },
            Instruction::StoreInRegister { x, nn } => {
                self.regs[x].0 = nn;
            },
            Instruction::AddToRegister { x, nn } => {
                self.regs[x].0 = self.regs[x].0.wrapping_add(nn);
            },
            Instruction::StoreRegister { x, y } => {
                self.regs[x] = self.regs[y];
            },
            Instruction::RegisterOr { x, y } => {
                self.regs[x].0 |= self.regs[y].0;
            },
            Instruction::RegisterAnd { x, y } => {
                self.regs[x].0 &= self.regs[y].0;
            },
            Instruction::RegisterXor { x, y } => {
                self.regs[x].0 ^= self.regs[y].0;
            },
            Instruction::AddRegisters { x, y } => {
                let (sum, carry) = u8::overflowing_add(
                    self.regs[x].0, self.regs[y].0
                );
                self.regs[x].0 = sum;
                self.regs[RegLabel::VF].0 = u8::from(carry);
            },
            Instruction::SubtractRegisters { x, y } => {
                let (diff, borrow) = u8::overflowing_sub(
                    self.regs[x].0, self.regs[y].0
                );
                self.regs[x].0 = diff;
                self.regs[RegLabel::VF].0 = u8::from(!borrow);
            },
            Instruction::RegisterShiftRight { x, y } => {
                self.regs[RegLabel::VF].0 = self.regs[y].0 & 0b1;
                self.regs[x].0 = self.regs[y].0 >> 1;
            },
            Instruction::SubtractFromRegister { x, y } => {
                let (diff, borrow) = u8::overflowing_sub(
                    self.regs[y].0, self.regs[x].0
                );
                self.regs[x].0 = diff;
                self.regs[RegLabel::VF].0 = u8::from(!borrow);
            },
            Instruction::RegisterShiftLeft { x, y } => {
                self.regs[RegLabel::VF].0 = (self.regs[y].0 & 0b1000_0000) >> 7;
                self.regs[x].0 = self.regs[y].0 << 1;
            },
            Instruction::SkipIfRegistersNotEqual { x, y } => {
                if self.regs[x] != self.regs[y] {
                    self.increment_pc();
                }
            },
            Instruction::StoreAddress { nnn } => { self.i_register = nnn },
            Instruction::JumpWithOffset { nnn } => {
                let offset = self.regs[RegLabel::V0].0;
                self.pc = nnn + offset;
            },
            Instruction::DrawSprite { x, y, n } => { self.draw_sprite(x, y, n) },
            Instruction::LoadDelayTimer { x } => {
                self.regs[x].0 = self.delay_timer;
            },
            Instruction::SetDelayTimer { x } => {
                self.delay_timer = self.regs[x].0;
            },
            Instruction::SetSoundTimer { x } => {
                self.sound_timer = self.regs[x].0;
            },
            Instruction::AddToIRegister { x } => {
                self.i_register += self.regs[x].0;
            },
            Instruction::SetToPreloadedSpriteData { x } => {
                let digit = u16::from(self.regs[x].0 & 0x0F);
                let addr = crate::FONT_DATA_BASE_ADDRESS + digit * 0x5;
                self.i_register = Address(addr);
            },
            Instruction::StoreBCDToMemory { x } => {
                let val = self.regs[x].0;
                let hundreds = val / 100;
                let tens = (val % 100) / 10;
                let ones = val % 10;
                self.memory[self.i_register] = hundreds;
                self.memory[self.i_register + 1] = tens;
                self.memory[self.i_register + 2] = ones;
            },
            Instruction::StoreToMemory { x } => {
                for i in 0..=x.as_number() {
                    let dest = self.i_register + i;
                    self.memory[dest] = self.regs[RegLabel::new(i)].0;
                }
                self.i_register += x.as_number() + 1;
            },
            Instruction::LoadFromMemory { x } => {
                for i in 0..=x.as_number() {
                    let src = self.i_register + i;
                    self.regs[RegLabel::new(i)].0 = self.memory[src];
                }
                self.i_register += x.as_number() + 1;
            },
            Instruction::RandomWithMask { x, nn } => {
                self.regs[x].0 = self.randomness_provider.random() & nn;
            },
            Instruction::SkipIfKeyPressed { x } => {
                let key = self.regs[x].0 & 0xF;
                let is_pressed = self.keyboard_provider.is_key_pressed(key);
                if is_pressed {
                    self.increment_pc();
                }
            },
            Instruction::SkipIfKeyNotPressed { x } => {
                let key = self.regs[x].0 & 0xF;
                let is_pressed = self.keyboard_provider.is_key_pressed(key);
                if !is_pressed {
                    self.increment_pc();
                }
            },
            Instruction::WaitForKeypress { .. } => todo!()
        }
    }

    fn draw_sprite(&mut self, x: RegLabel, y: RegLabel, n: u8) {
        let base_x = self.regs[x].0;
        let base_y = self.regs[y].0;
        let i = usize::from(self.i_register.0);
        let sprite_data = &self.memory.0[i..][..usize::from(n)];
        let mut any_unset = false;
        for (i, &byte) in sprite_data.iter().enumerate() {
            let i = u8::try_from(i).unwrap();
            let x = usize::from(base_x);
            let y = usize::from(base_y + i);
            let screen_value = &mut self.graphics.0[y][x];
            *screen_value ^= byte;
            if (*screen_value & byte) != 0 {
                any_unset = true;
            }
        }
        self.regs[RegLabel::VF].0 = u8::from(any_unset);
    }
}

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
}

/// A type that can provide time to the [`Chip8Interpreter`].
///
/// It processes "ticks" that are at a rate of 60 Hz.
pub trait TimeProvider {
    /// Sets the internal "epoch" used by this `TimeProvider` to the current moment.
    fn set_epoch(&mut self);

    /// Pauses the current thread until the next tick has occurred.
    fn wait_until_next_tick(&mut self);
}
