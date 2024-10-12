mod chip_core;

use minifb::{Key, Window, WindowOptions};
use chip_core::{ ChipCore };

const FRAMEBUFFER_SIZE: usize = ChipCore::SCR_WIDTH * ChipCore::SCR_HEIGHT;
const WINDOW_SCALE: usize = 12;

fn main() {
    let mut buffer: [u32; FRAMEBUFFER_SIZE] = [0; FRAMEBUFFER_SIZE];

    let window_options = WindowOptions {
        scale_mode: minifb::ScaleMode::Stretch,
        .. WindowOptions::default()
    };

    let mut window = Window::new("ChipRust8",
                                 ChipCore::SCR_WIDTH * WINDOW_SCALE,
                                 ChipCore::SCR_HEIGHT * WINDOW_SCALE, window_options)
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    window.set_target_fps(60);

    while window.is_open() {
        window.update_with_buffer(&buffer, ChipCore::SCR_WIDTH, ChipCore::SCR_HEIGHT).unwrap();
    }
}