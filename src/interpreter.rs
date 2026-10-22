//! The CHIP-8 interpreter.

use std::sync::{Arc, Mutex};

use crate::{Address, GraphicsState, Memory, RegLabel, Registers, inst::Instruction};

pub mod providers;
use providers::{RandomnessProvider, KeyboardProvider, SoundProvider, TimeProvider};

/// The container for all the information stored in the CHIP-8 interpreter.
pub struct Chip8Interpreter {
    memory: Memory,
    regs: Registers,
    i_register: Address,
    graphics: Arc<Mutex<GraphicsState>>,
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<Address>,
    /// Program Counter
    pc: Address,
    /// Are we currently waiting for a keypress? (Instruction FX0A)
    ///
    /// If so, what register are we going to store the keypress into?
    waiting_for_keypress: Option<RegLabel>,
    randomness_provider: Box<dyn RandomnessProvider + Send + Sync>,
    keyboard_provider: Box<dyn KeyboardProvider + Send + Sync>,
    sound_provider: Box<dyn SoundProvider + Send + Sync>,
    time_provider: Box<dyn TimeProvider + Send + Sync>,
    last_tick_known: u64,
}

impl Chip8Interpreter {
    /// Initializes a new interpreter with no loaded program.
    pub fn new<R, K, S, T>(
        randomness_provider: R, keyboard_provider: K,
        sound_provider: S, time_provider: T,
    ) -> Self
    where
        R: RandomnessProvider + Send + Sync + 'static,
        K: KeyboardProvider + Send + Sync + 'static,
        S: SoundProvider + Send + Sync + 'static,
        T: TimeProvider + Send + Sync + 'static,
    {
        Chip8Interpreter {
            memory: Memory::new(),
            regs: Registers::new(),
            i_register: Address::new(0),
            graphics: Arc::new(Mutex::new(GraphicsState::blank())),
            delay_timer: 0,
            sound_timer: 0,
            stack: Vec::with_capacity(16),
            pc: Address::new(0x200),
            waiting_for_keypress: None,
            randomness_provider: Box::new(randomness_provider),
            keyboard_provider: Box::new(keyboard_provider),
            sound_provider: Box::new(sound_provider),
            time_provider: Box::new(time_provider),
            last_tick_known: 0,
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

    /// Returns a handle to the [`GraphicsState`] of this [`Chip8Interpreter`].
    ///
    /// This lock should be kept locked for as little time as possible! Sprite
    /// code within the interpreter must **wait** for this Mutex to be unlocked.
    pub fn graphics_handle(&self) -> Arc<Mutex<GraphicsState>> {
        Arc::clone(&self.graphics)
    }

    /// Starts the interpreter. **This function does not exit.**
    pub fn boot(&mut self) {
        self.last_tick_known = self.time_provider.current_tick_count();
        loop {
            self.drive();
        }
    }

    /// Drives the interpreter--executes instructions, processes input, etc.
    fn drive(&mut self) {
        if let Some(x) = self.waiting_for_keypress {
            let pressed = self.keyboard_provider.is_any_key_pressed();
            if pressed == 255 {
                return;
            }
            self.regs[x].0 = pressed;
            self.waiting_for_keypress = None;
        }
        let inst_hi = self.memory[self.pc];
        let inst_lo = self.memory[self.pc + 1];
        let Ok(inst) = Instruction::parse(inst_hi, inst_lo) else {
            let n = u16::from_be_bytes([inst_hi, inst_lo]);
            eprintln!("Unrecognized instruction {n:#06X}");
            return;
        };
        dbg!(inst);
        self.exec_instruction(inst);

        let current_tick = self.time_provider.current_tick_count();
        if current_tick > self.last_tick_known {
            self.last_tick_known = current_tick;
            self.delay_timer = self.delay_timer.saturating_sub(1);
            self.sound_timer = self.sound_timer.saturating_sub(1);
        }

        if self.sound_timer > 0 {
            self.sound_provider.enable_sound();
        } else {
            self.sound_provider.disable_sound();
        }
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
    #[expect(clippy::too_many_lines)]
    fn exec_instruction(&mut self, inst: Instruction) {
        self.increment_pc();
        match inst {
            Instruction::ClearScreen => {
                let mut guard = self.graphics.lock().unwrap();
                *guard = GraphicsState::blank();
            },
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
            Instruction::WaitForKeypress { x } => {
                let pressed = self.keyboard_provider.is_any_key_pressed();
                #[expect(clippy::if_not_else)]
                if pressed != 255 {
                    self.regs[x].0 = pressed;
                } else {
                    self.waiting_for_keypress = Some(x);
                }
            },
        }
    }

    fn draw_sprite(&mut self, x: RegLabel, y: RegLabel, n: u8) {
        let mut graphics = self.graphics.lock().unwrap();
        let base_x = self.regs[x].0;
        let base_y = self.regs[y].0;
        let i = usize::from(self.i_register.0);
        let sprite_data = &self.memory.0[i..][..usize::from(n)];
        let mut any_unset = false;
        for (i, &byte) in sprite_data.iter().enumerate() {
            let i = u8::try_from(i).unwrap();
            let x_bit = usize::from(base_x);
            let y_bit = usize::from(base_y + i);

            let x_byte_base = x_bit / 8;
            let bits_into_x_byte = x_bit % 8;
            let x_segment = &mut graphics.0[y_bit][x_byte_base..][..2];
            let left_byte = byte >> bits_into_x_byte;
            let right_byte = u8::unbounded_shl(byte, (8 - bits_into_x_byte).try_into().unwrap());
            x_segment[0] ^= left_byte;
            x_segment[1] ^= right_byte;
            if ((x_segment[0] & left_byte) | (x_segment[1] & right_byte)) != 0 {
                any_unset = true;
            }
        }
        self.regs[RegLabel::VF].0 = u8::from(any_unset);
    }
}
