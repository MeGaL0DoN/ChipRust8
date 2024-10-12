mod chip_core;

use std::path::Path;

use minifb::{Key, Window, WindowOptions};
use chip_core::{ ChipCore };

const FRAMEBUFFER_SIZE: usize = ChipCore::SCR_WIDTH * ChipCore::SCR_HEIGHT;
const WINDOW_SCALE: usize = 12;
const IPF: u16 = 10;

fn main() {
    let mut chip = ChipCore::new();
    chip.load_rom(Path::new("testrom.ch8"));

    let mut screen_buf: [u32; FRAMEBUFFER_SIZE] = [0; FRAMEBUFFER_SIZE];

    let mut window =
        Window::new("ChipRust8",
                    ChipCore::SCR_WIDTH * WINDOW_SCALE, ChipCore::SCR_HEIGHT * WINDOW_SCALE, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    window.set_target_fps(60);

    while window.is_open() {
        for i in 0..IPF {
            chip.execute();
        }

        chip.render_to_rgb_buffer(&mut screen_buf);
        window.update_with_buffer(&screen_buf, ChipCore::SCR_WIDTH, ChipCore::SCR_HEIGHT).unwrap();
    }
}