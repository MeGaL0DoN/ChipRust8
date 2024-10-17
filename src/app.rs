use crate::chip_core::{ ChipCore };
use std::path::PathBuf;
use std::time::{ Duration, Instant };
use minifb::{ Key, KeyRepeat, Menu, Window, WindowOptions };

pub struct App {
    chip: ChipCore,
    chip_screen_buf: [u32; ChipCore::CHIP_FRAMEBUFFER_SIZE],
    schip_screen_buf: [u32; ChipCore::SCHIP_FRAMEBUFFER_SIZE],
    window: Window,
    options_menu : Menu,
    file_menu : Menu,
    rom_path: PathBuf,
    rom_loaded: bool,
    chip_paused: bool,
    ipf: u32,
    execute_times: f64,
    execute_count: u32,
    seconds_timer: Instant
}

impl App {
    pub const APP_NAME: &'static str = "ChipRust8";
    const WINDOW_SCALE: usize = 12;
    const FILE_MENU_LOAD_ID: usize = 1;
    const FILE_MENU_RELOAD_ID: usize = 2;
    const KEY_BINDING: [Key; 16] = [
        Key::X, Key::Key1, Key::Key2, Key::Key3, Key::Q, Key::W, Key::E, Key::A,
        Key::S, Key::D, Key::Z, Key::C, Key::Key4, Key::R, Key::F, Key::V,
    ];

    fn update_chip_input(&mut self) {
        let ipf_step = if self.window.is_key_down(Key::RightShift) || self.window.is_key_down(Key::LeftShift) { 100000 } else { 1 };

        if self.window.is_key_pressed(Key::Right, KeyRepeat::Yes) {
            self.ipf += ipf_step;
            self.update_window_title();
        }
        else if self.window.is_key_pressed(Key::Left, KeyRepeat::Yes) && self.ipf > ipf_step {
            self.ipf -= ipf_step;
            self.update_window_title();
        }

        for i in 0..16 {
            let new_key_state = self.window.is_key_down(Self::KEY_BINDING[i]);

            if self.chip.get_keys()[i] != new_key_state {
                self.chip.key_event(i as u8, new_key_state);
            }
        }
    }

    pub fn new() -> Self {
        let mut app = Self {
            chip: ChipCore::new(),
            chip_screen_buf: [0; ChipCore::CHIP_FRAMEBUFFER_SIZE],
            schip_screen_buf: [0; ChipCore::SCHIP_FRAMEBUFFER_SIZE],

            window: Window::new(Self::APP_NAME, ChipCore::CHIP_SCR_WIDTH * App::WINDOW_SCALE,
                                ChipCore::CHIP_SCR_HEIGHT * App::WINDOW_SCALE, WindowOptions::default())
                .unwrap_or_else(|e| {
                    panic!("{}", e);
                }),

            options_menu: Menu::new("Options").unwrap(),
            file_menu: Menu::new("File").unwrap(),

            rom_path: std::env::current_dir().unwrap(),
            rom_loaded: false,
            chip_paused: false,
            ipf: 11,
            execute_times: 0.0,
            execute_count: 0,
            seconds_timer: Instant::now(),
        };

        app.file_menu.add_item("Load", Self::FILE_MENU_LOAD_ID).build();
        app.file_menu.add_item("Reload", Self::FILE_MENU_RELOAD_ID).build();

        app.window.add_menu(&app.file_menu);
        app.window.add_menu(&app.options_menu);

        app.window.set_target_fps(60);
        app
    }

    fn update_window_title(&mut self) {
        let title = if self.chip_paused {
            format!("{} (Paused)", Self::APP_NAME)
        }
        else {
            format!("{} (IPF: {})", Self::APP_NAME, self.ipf)
        };

        self.window.set_title(title.as_str());
    }

    fn check_seconds_timer(&mut self) {
        if self.seconds_timer.elapsed() >= Duration::from_secs(1) && self.rom_loaded && !self.chip_paused {
            let avg_exec_time = (self.execute_times / self.execute_count as f64) * 1000.0;
            println!("Average execute time: {:.3} ms", avg_exec_time);

            self.execute_times = 0.0;
            self.execute_count = 0;
            self.seconds_timer = Instant::now();
        }
    }

    fn load_rom(&mut self) {
        if self.chip.load_rom(self.rom_path.as_path()) {
            self.rom_loaded = true;
            self.chip_paused = false;
            self.update_window_title();
        }
    }

    fn file_load_dialog(&mut self) {
        let res = rfd::FileDialog::new()
            .add_filter("Chip8 ROM", &["ch8", "bnc"])
            .set_directory(&self.rom_path)
            .pick_files();

        if let Some(paths) = res {
            if self.chip.load_rom(paths[0].as_path()) {
                self.rom_path = paths[0].clone();
                self.load_rom();
            }
        }
    }

    fn update_window(&mut self) {
        if let Some(menu_id) = self.window.is_menu_pressed() {
            match menu_id {
                Self::FILE_MENU_LOAD_ID => {
                    self.file_load_dialog();
                }
                Self::FILE_MENU_RELOAD_ID => {
                    if self.rom_loaded {
                        self.load_rom();
                    }
                }
                _ => {}
            }
        }

        if self.chip.high_res_mode() {
            self.window.update_with_buffer(&self.schip_screen_buf, ChipCore::SCHIP_SCR_WIDTH, ChipCore::SCHIP_SCR_HEIGHT).unwrap();
        }
        else {
            self.window.update_with_buffer(&self.chip_screen_buf, ChipCore::CHIP_SCR_WIDTH, ChipCore::CHIP_SCR_HEIGHT).unwrap();
        }
    }

    pub fn run(&mut self) {
        while self.window.is_open() {
            if self.window.is_key_pressed(Key::Escape, KeyRepeat::No) {
                self.file_load_dialog();
            }

            if self.rom_loaded {
                if self.window.is_key_pressed(Key::Tab, KeyRepeat::No) {
                    self.chip_paused = !self.chip_paused;
                    self.update_window_title();
                }

                if !self.chip_paused {
                    self.update_chip_input();
                    self.chip.update_timers();

                    let execute_start = Instant::now();

                    for _ in 0..self.ipf {
                        self.chip.execute();
                    }

                    self.execute_times += execute_start.elapsed().as_secs_f64();
                    self.execute_count += 1;

                    if self.chip.high_res_mode() {
                        self.chip.render_to_rgb_schip_buffer(&mut self.schip_screen_buf);
                    }
                    else {
                        self.chip.render_to_rgb_chip_buffer(&mut self.chip_screen_buf);
                    }
                }
            }

            self.check_seconds_timer();
            self.update_window();
        }
    }
}