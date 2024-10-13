mod chip_core;

use minifb::{ Key, Window, Menu, WindowOptions };
use chip_core::{ ChipCore };

const FRAMEBUFFER_SIZE: usize = ChipCore::SCR_WIDTH * ChipCore::SCR_HEIGHT;
const WINDOW_SCALE: usize = 12;
const FILE_MENU_LOAD_ID: usize = 1;
const FILE_MENU_RELOAD_ID: usize = 2;
const IPF: u16 = 11;

fn update_chip_input(window: &Window, chip: &mut ChipCore) {
    let new_keys = [
        window.is_key_down(Key::X),
        window.is_key_down(Key::Key1),
        window.is_key_down(Key::Key2),
        window.is_key_down(Key::Key3),
        window.is_key_down(Key::Q),
        window.is_key_down(Key::W),
        window.is_key_down(Key::E),
        window.is_key_down(Key::A),
        window.is_key_down(Key::S),
        window.is_key_down(Key::D),
        window.is_key_down(Key::Z),
        window.is_key_down(Key::C),
        window.is_key_down(Key::Key4),
        window.is_key_down(Key::R),
        window.is_key_down(Key::F),
        window.is_key_down(Key::V),
    ];

    for i in 0..16 {
        if chip.get_keys()[i] != new_keys[i] {
            chip.key_event(i as u8, new_keys[i]);
        }
    }
}

fn main() {
    let mut chip = ChipCore::new();
    let mut screen_buf: [u32; FRAMEBUFFER_SIZE] = [0; FRAMEBUFFER_SIZE];

    let mut window =
        Window::new("ChipRust8",
                    ChipCore::SCR_WIDTH * WINDOW_SCALE, ChipCore::SCR_HEIGHT * WINDOW_SCALE, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    window.set_target_fps(60);

    let options_menu = Menu::new("Options").unwrap();
    let mut file_menu = Menu::new("File").unwrap();
    file_menu.add_item("Load", FILE_MENU_LOAD_ID).build();
    file_menu.add_item("Reload", FILE_MENU_RELOAD_ID).build();

    window.add_menu(&file_menu);
    window.add_menu(&options_menu);

    let mut last_path = std::env::current_dir().unwrap();
    let mut rom_loaded = false;

    while window.is_open() {
        if let Some(menu_id) = window.is_menu_pressed() {
            match menu_id {
                FILE_MENU_LOAD_ID => {
                    let res = rfd::FileDialog::new()
                        .add_filter("Chip8 ROM", &["ch8", "bnc"])
                        .set_directory(&last_path)
                        .pick_files();

                    if let Some(paths) = res {
                        if chip.load_rom(paths[0].as_path()) {
                            rom_loaded = true;
                            last_path = paths[0].clone();
                        }
                    }
                }
                FILE_MENU_RELOAD_ID => {
                    if rom_loaded {
                        chip.load_rom(last_path.as_path());
                    }
                }
                _ => {}
            }
        }

        if rom_loaded {
            update_chip_input(&window, &mut chip);
            chip.update_timers();

            for _ in 0..IPF {
                chip.execute();
            }

            chip.render_to_rgb_buffer(&mut screen_buf);
        }

        window.update_with_buffer(&screen_buf, ChipCore::SCR_WIDTH, ChipCore::SCR_HEIGHT).unwrap();
    }
}