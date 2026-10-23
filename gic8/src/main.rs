//! GIC8 (Graphical Interpreter for Chip-8).
//!
//! Brought to you by [`crabchip8`].

use std::{num::NonZeroU32, sync::{Arc, Mutex}};

use crabchip8::{
    GraphicsState,
    interpreter::{Chip8Interpreter, providers},
};
use winit::{
    application::ApplicationHandler,
    dpi,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};
use softbuffer::Surface;

type WindowSurface = Surface<Arc<Window>, Arc<Window>>;

struct App {
    c8_graphics: Arc<Mutex<GraphicsState>>,
    win: Option<WindowState>,
}

struct WindowState {
    window: Arc<Window>,
    surface: WindowSurface,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.win.get_or_insert_with(|| {
            let attrs = Window::default_attributes()
                .with_title("GIC8")
                .with_inner_size(dpi::LogicalSize::new(640, 320))
                .with_resize_increments(dpi::LogicalSize::new(64, 32));
            let window = event_loop.create_window(attrs)
                .expect("failed to create window");
            let window = Arc::new(window);
            let ctx = softbuffer::Context::new(Arc::clone(&window))
                .expect("failed to create softbuffer::Context");
            let surface = Surface::new(&ctx, Arc::clone(&window))
                .expect("failed to create softbuffer::Surface");
            WindowState { window, surface }
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(win) = &mut self.win else { return };
        if window_id != win.window.id() { return };
        match event {
            WindowEvent::RedrawRequested => {
                let graphics_state = {
                    let guard = self.c8_graphics.lock().unwrap();
                    GraphicsState::clone(&*guard)
                };
                draw(&graphics_state, &win.window, &mut win.surface);
                win.window.request_redraw();
            },
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {},
        }
    }
}

fn draw(graphics: &GraphicsState, window: &Window, surface: &mut WindowSurface) {
    const WHITE: u32 = 0x00FFFFFF;
    const BLACK: u32 = 0x00181818;

    let dpi::PhysicalSize { width, height } = window.inner_size();
    let scaled_width = width / 64;
    let scaled_height = height / 32;
    let pixel_scale = Ord::min(scaled_width, scaled_height);
    surface.resize(
        NonZeroU32::new(pixel_scale * 64).expect("window width should be non-zero"),
        NonZeroU32::new(pixel_scale * 32).expect("window height should be non-zero"),
    ).expect("resizing surface should not fail");
    let mut buffer = surface.buffer_mut().expect("getting softbuffer::Buffer");
    assert_eq!(buffer.width().get() % 64, 0);
    assert_eq!(buffer.height().get() % 32, 0);
    let pixel_scale = usize::try_from(pixel_scale).unwrap();
    let row_len = 64 * pixel_scale;

    for (y, row) in graphics.pixel_iterator().enumerate() {
        for y_subpixel in 0..pixel_scale {
            let row_index = (y * pixel_scale + y_subpixel) * row_len;
            for (x, pixel) in row.into_iter().enumerate() {
                let color = if pixel { WHITE } else { BLACK };
                for x_subpixel in 0..pixel_scale {
                    buffer[row_index + (x * pixel_scale) + x_subpixel] = color;
                }
            }
        }
    }

    buffer.present().expect("failed presenting buffer");
}

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
    let mut app = App {
        c8_graphics: graphics,
        win: None,
    };

    let event_loop = EventLoop::new().expect("failed to construct event loop");
    let _ = event_loop.run_app(&mut app);
    // std::thread::sleep(std::time::Duration::from_millis(200));
    // let graphics = GraphicsState::clone(&graphics.lock().unwrap());
    // debug_print_graphics(&graphics);
}

#[expect(dead_code)]
fn debug_print_graphics(graphics: &GraphicsState) {
    for row in graphics.pixel_iterator() {
        for pixel in row {
            if pixel {
                print!("⠿");
            } else {
                print!(" ");
            }
        }
        println!()
    }
}
