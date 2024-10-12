mod chip_core;

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 64 * 12;
const HEIGHT: usize = 32 * 12;

fn main() {
    let mut buffer: [u32; 64 * 32] = [0; 64 * 32];

    let window_options = WindowOptions {
        scale_mode: minifb::ScaleMode::Stretch,
        .. WindowOptions::default()
    };

    let mut window = Window::new("ChipRust8", WIDTH, HEIGHT, window_options)
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    while window.is_open() {
        window.update_with_buffer(&buffer, 64, 32).unwrap();
    }
}