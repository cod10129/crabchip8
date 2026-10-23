//! GIC8 (Graphical Interpreter for Chip-8).
//!
//! Brought to you by [`crabchip8`].

use crabchip8::{
    GraphicsState,
    interpreter::{Chip8Interpreter, providers},
};

fn main() {
    let program = std::fs::read("program.ch8").unwrap();
    let mut interpreter = Chip8Interpreter::new(
        providers::HashRandomnessProvider::new(),
        providers::DummyKeyboardProvider,
        providers::DummySoundProvider::new(),
        providers::StdTimeProvider::new(),
    );
    interpreter.load_program(&program);
    let graphics = interpreter.graphics_handle();
    std::thread::spawn(move || {
        interpreter.boot();
    });
    std::thread::sleep(std::time::Duration::from_millis(200));
    let graphics = GraphicsState::clone(&graphics.lock().unwrap());
    debug_print_graphics(&graphics);
}

fn debug_print_graphics(graphics: &GraphicsState) {
    for row in graphics.data() {
        for byte in row {
            for i in 0..8 {
                let bit = (byte >> (7 - i)) & 0b1;
                if bit > 0 {
                    print!("⠿");
                } else {
                    print!(" ");
                }
            }
        }
        println!();
    }
}
