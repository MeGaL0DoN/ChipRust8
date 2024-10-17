use std::path::Path;
use std::{fs};
use rand::Rng;
use rand::rngs::ThreadRng;

pub struct ChipCore {
    screen_buf: [u64; ChipCore::CHIP_SCR_HEIGHT],
    schip_screen_buf: [u128; ChipCore::SCHIP_SCR_HEIGHT],
    ram: [u8; ChipCore::RAM_SIZE],
    regs: [u8; 16],
    stack: [u16; 16],
    keys: [bool; 16],
    awaiting_key_release: bool,
    released_key_reg: i8,
    sp: u16,
    pc: u16,
    i_reg: u16,
    delay_timer: u8,
    sound_timer: u8,
    high_res_mode: bool,
    rng: ThreadRng,
}

impl ChipCore {
    pub const CHIP_SCR_WIDTH: usize = 64;
    pub const SCHIP_SCR_WIDTH: usize = Self::CHIP_SCR_WIDTH * 2;
    pub const CHIP_SCR_HEIGHT: usize = 32;
    pub const SCHIP_SCR_HEIGHT: usize = Self::CHIP_SCR_HEIGHT * 2;
    pub const RAM_SIZE: usize = 4096;
    pub const CHIP_FRAMEBUFFER_SIZE: usize = Self::CHIP_SCR_WIDTH * Self::CHIP_SCR_HEIGHT;
    pub const SCHIP_FRAMEBUFFER_SIZE: usize = Self::SCHIP_SCR_WIDTH * Self::SCHIP_SCR_HEIGHT;

    const CHIP_FONT: [u8; 80] =
    [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ];
    const SCHIP_FONT: [u8; 100] = [
        0x3C, 0x7E, 0xE7, 0xC3, 0xC3, 0xC3, 0xC3, 0xE7, 0x7E, 0x3C, // big 0
        0x18, 0x38, 0x58, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, // big 1
        0x3E, 0x7F, 0xC3, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xFF, 0xFF, // big 2
        0x3C, 0x7E, 0xC3, 0x03, 0x0E, 0x0E, 0x03, 0xC3, 0x7E, 0x3C, // big 3
        0x06, 0x0E, 0x1E, 0x36, 0x66, 0xC6, 0xFF, 0xFF, 0x06, 0x06, // big 4
        0xFF, 0xFF, 0xC0, 0xC0, 0xFC, 0xFE, 0x03, 0xC3, 0x7E, 0x3C, // big 5
        0x3E, 0x7C, 0xE0, 0xC0, 0xFC, 0xFE, 0xC3, 0xC3, 0x7E, 0x3C, // big 6
        0xFF, 0xFF, 0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x60, // big 7
        0x3C, 0x7E, 0xC3, 0xC3, 0x7E, 0x7E, 0xC3, 0xC3, 0x7E, 0x3C, // big 8
        0x3C, 0x7E, 0xC3, 0xC3, 0x7F, 0x3F, 0x03, 0x03, 0x3E, 0x7C  // big 9
    ];
    const SCHIP_FONT_OFFSET: usize = Self::CHIP_FONT.len();

    pub fn new() -> Self {
        let mut chip_core = Self {
            screen_buf: [0; Self::CHIP_SCR_HEIGHT],
            schip_screen_buf: [0; Self::SCHIP_SCR_HEIGHT],
            ram: [0; Self::RAM_SIZE],
            regs: [0; 16],
            stack: [0; 16],
            keys: [false; 16],
            awaiting_key_release: false,
            released_key_reg: -1,
            sp: 0,
            pc: 0x200,
            i_reg: 0,
            delay_timer: 0,
            sound_timer: 0,
            high_res_mode: false,
            rng: rand::thread_rng(),
        };

        chip_core.ram[..Self::CHIP_FONT.len()].copy_from_slice(&Self::CHIP_FONT);
        chip_core.ram[Self::SCHIP_FONT_OFFSET..Self::SCHIP_FONT.len() + Self::SCHIP_FONT_OFFSET].copy_from_slice(&Self::SCHIP_FONT);

        chip_core
    }

    pub fn load_rom(&mut self, path: &Path) -> bool {
        let metadata: fs::Metadata = match fs::metadata(path) {
            Ok(meta) => meta,
            Err(_) => return false,
        };

        if metadata.len() as usize <= Self::RAM_SIZE - 0x200 {
            let bytes = match fs::read(path) {
                Ok(val) => val,
                Err(_) => return false,
            };

            *self = Self::new();
            self.ram[0x200..0x200 + bytes.len()].copy_from_slice(&bytes);
            return true;
        }
        false
    }

    pub fn render_to_rgb_chip_buffer(&mut self, buf: &mut [u32]) {
        for i in 0..Self::CHIP_FRAMEBUFFER_SIZE {
            buf[i] = if ((self.screen_buf[i >> 6] >> (Self::CHIP_SCR_WIDTH - 1 - (i & 0x3F))) & 0x1) == 1 { 0xFFFFFFFF } else { 0 };
        }
    }
    pub fn render_to_rgb_schip_buffer(&mut self, buf: &mut[u32]) {
        for i in 0..Self::SCHIP_FRAMEBUFFER_SIZE {
            buf[i] = if ((self.schip_screen_buf[i >> 7] >> (Self::SCHIP_SCR_WIDTH - 1 - (i & 0x7F))) & 0x1) == 1 { 0xFFFFFFFF } else { 0 };
        }
    }

    pub fn key_event(&mut self, key: u8, action: bool) {
        self.keys[(key & 0xF) as usize] = action;

        if self.awaiting_key_release && !action  {
            self.regs[self.released_key_reg as usize] = key;
            self.released_key_reg = -1;
        }
    }
    pub fn get_keys(&self) -> &[bool; 16] {
        &self.keys
    }

    pub fn high_res_mode(&self) -> bool {
        self.high_res_mode
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
    
    fn shift_screenbuf_down<T, const N: usize>(buf: &mut [T; N], num_pixels: usize) where T: Copy + Default  {
        for i in (num_pixels..N).rev() {
            buf[i] = buf[i - num_pixels];
        }

        for elem in buf.iter_mut().take(num_pixels) {
            *elem = T::default();
        }
    }
    pub fn execute(&mut self) {
        let opcode= ((self.ram[(self.pc & 0xFFF) as usize] as u16) << 8) | (self.ram[((self.pc + 1) & 0xFFF) as usize] as u16);
        self.pc += 2;

        let x = || -> usize { ((opcode & 0x0F00) >> 8) as usize };
        let y = || -> usize { ((opcode & 0x00F0) >> 4) as usize };
        let data = || -> u8 { (opcode & 0x00FF) as u8 };
        let addr = || -> u16 { opcode & 0x0FFF };

        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x00E0 => {
                        if self.high_res_mode {
                            self.schip_screen_buf.fill(0);
                        }
                        else {
                            self.screen_buf.fill(0);
                        }
                    }
                    0x00EE => {
                        self.sp = self.sp.wrapping_sub(1) & 0xF;
                        self.pc = self.stack[self.sp as usize];
                    }
                    0x00FE => {
                        if self.high_res_mode {
                            self.high_res_mode = false;
                            self.screen_buf.fill(0);   
                        }
                    }
                    0x00FF => {
                        if !self.high_res_mode {
                            self.high_res_mode = true;
                            self.schip_screen_buf.fill(0);   
                        }
                    }
                    0x00FB => {
                        if self.high_res_mode {
                            for row in self.schip_screen_buf.iter_mut() {
                                *row >>= 4;
                            }
                        }
                        else {
                            for row in self.screen_buf.iter_mut() {
                                *row >>= 4;
                            }
                        }
                    }
                    0x00FC => {
                        if self.high_res_mode {
                            for row in self.schip_screen_buf.iter_mut() {
                                *row <<= 4;
                            }
                        }
                        else {
                            for row in self.screen_buf.iter_mut() {
                                *row <<= 4;
                            }
                        }
                    }
                    _ => {
                        match opcode & 0xFFF0 {
                            0x00C0 => {
                                let n = (opcode & 0x000F) as usize;
                                
                                if self.high_res_mode {
                                    Self::shift_screenbuf_down(&mut self.schip_screen_buf, n);
                                }
                                else {
                                    Self::shift_screenbuf_down(&mut self.screen_buf, n);
                                }
                            }
                            _ => {
                                println!("Unknown opcode {:X}", opcode);
                            }
                        }
                    }
                }
            }
            0x1000 => {
                self.pc = addr();
            }
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp = (self.sp + 1) & 0xF;
                self.pc = addr();
            }
            0x3000 => {
                if self.regs[x()] == data() {
                    self.pc += 2;
                }
            }
            0x4000 => {
                if self.regs[x()] != data() {
                    self.pc += 2;
                }
            }
            0x5000 => {
                match opcode & 0x000F {
                    0x0000 => {
                        if self.regs[x()] == self.regs[y()] {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        println!("Unknown opcode {:X}", opcode);
                    }
                }
            }
            0x6000 => {
                self.regs[x()] = data();
            }
            0x7000 => {
                self.regs[x()] = self.regs[x()].wrapping_add(data());
            }
            0x8000 => {
                match opcode & 0x000F {
                    0x0000 => {
                        self.regs[x()] = self.regs[y()];
                    }
                    0x0001 => {
                        self.regs[x()] |= self.regs[y()];
                        self.regs[0xF] = 0; // VF reset quirk.
                    }
                    0x0002 => {
                        self.regs[x()] &= self.regs[y()];
                        self.regs[0xF] = 0; // VF reset quirk.
                    }
                    0x0003 => {
                        self.regs[x()] ^= self.regs[y()];
                        self.regs[0xF] = 0; // VF reset quirk.
                    }
                    0x0004 => {
                        let (res, overflow) = self.regs[x()].overflowing_add(self.regs[y()]);
                        self.regs[x()] = res;
                        self.regs[0xF] = overflow as u8;
                    }
                    0x0005 => {
                        let (res, overflow) = self.regs[x()].overflowing_sub(self.regs[y()]);
                        self.regs[x()] = res;
                        self.regs[0xF] = !overflow as u8;
                    }
                    0x0006 => {
                        let shifted = self.regs[x()] & 0x1;
                        self.regs[x()] >>= 1;
                        self.regs[0xF] = shifted;
                    }
                    0x0007 => {
                        let (res, overflow) = self.regs[y()].overflowing_sub(self.regs[x()]);
                        self.regs[x()] = res;
                        self.regs[0xF] = !overflow as u8;
                    }
                    0x000E => {
                        let shifted = (self.regs[x()] & 0x80) >> 7;
                        self.regs[x()] <<= 1;
                        self.regs[0xF] = shifted;
                    }
                    _ => {
                        println!("Unknown opcode {:X}", opcode);
                    }
                }
            }
            0x9000 => {
                match opcode & 0x000F {
                    0x0000 => {
                        if self.regs[x()] != self.regs[y()] {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        println!("Unknown opcode {:X}", opcode);
                    }
                }
            }
            0xA000 => {
                self.i_reg = addr();
            }
            0xB000 => {
                self.pc = (self.regs[0] as u16) + addr();
            }
            0xC000 => {
                self.regs[x()] = self.rng.gen::<u8>() & data();
            }
            0xD000 => {
                Self::dxyn(self, opcode);
            }
            0xE000 => {
                match opcode & 0x00FF {
                    0x009E => {
                        if self.keys[(self.regs[x()] & 0xF) as usize] {
                            self.pc += 2;
                        }
                    }
                    0x00A1 => {
                        if !self.keys[(self.regs[x()] & 0xF) as usize] {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        println!("Unknown opcode {:X}", opcode);
                    }
                }
            }
            0xF000 => {
                match opcode & 0x00FF {
                    0x0007 => {
                        self.regs[x()] = self.delay_timer;
                    }
                    0x000A => {
                        if !self.awaiting_key_release {
                            self.awaiting_key_release = true;
                            self.released_key_reg = x() as i8;
                        }
                        else if self.released_key_reg == -1 {
                            self.awaiting_key_release = false;
                            return;
                        }

                        self.pc -= 2;
                    }
                    0x0015 => {
                        self.delay_timer = self.regs[x()];
                    }
                    0x0018 => {
                        self.sound_timer = self.regs[x()];
                    }
                    0x001E => {
                        self.i_reg = self.i_reg.wrapping_add(self.regs[x()] as u16);
                    }
                    0x0029 => {
                        self.i_reg = ((self.regs[x()] & 0xF) * 0x5) as u16;
                    }
                    0x0030 => {
                        self.i_reg = Self::CHIP_FONT.len() as u16 + ((self.regs[x()] & 0xF) * 10) as u16;
                    }
                    0x0033 => {
                        self.ram[self.i_reg as usize & 0xFFF] = self.regs[x()] / 100;
                        self.ram[(self.i_reg as usize + 1) & 0xFFF] = (self.regs[x()] / 10) % 10;
                        self.ram[(self.i_reg as usize + 2) & 0xFFF] = self.regs[x()] % 10;
                    }
                    0x0055 => {
                        for i in 0..=x() {
                            self.ram[(self.i_reg as usize + i) & 0xFFF] = self.regs[i];
                        }
                    }
                    0x0065 => {
                        for i in 0..=x() {
                            self.regs[i] = self.ram[(self.i_reg as usize + i) & 0xFFF];
                        }
                    }
                    _ => {
                        println!("Unknown opcode {:X}", opcode);
                    }
                }
            }
            _ => {
                println!("Unknown opcode {:X}", opcode);
            }
        }
    }

    fn dxyn(&mut self, opcode: u16) {
        let mut x_pos = self.regs[((opcode & 0x0F00) >> 8) as usize];
        let mut y_pos = self.regs[((opcode & 0x00F0) >> 4) as usize];
        let height = opcode & 0x000F;

        self.regs[0xF] = 0;

        if !self.high_res_mode {
            x_pos %= Self::CHIP_SCR_WIDTH as u8;
            y_pos %= Self::CHIP_SCR_HEIGHT as u8;

            if height == 0 {
                self.draw_lores_sprite::<true>(x_pos, y_pos, height);
            }
            else {
                self.draw_lores_sprite::<false>(x_pos, y_pos, height);
            }
        }
        else {
            x_pos %= Self::SCHIP_SCR_WIDTH as u8;
            y_pos %= Self::SCHIP_SCR_HEIGHT as u8;

            if height == 0 {
                self.draw_hires_sprite::<true>(x_pos, y_pos, height);
            }
            else {
                self.draw_hires_sprite::<false>(x_pos, y_pos, height);
            }
        }
    }
    
    fn draw_lores_sprite<const DOUBLE_HEIGHT: bool>(&mut self, x_pos: u8, mut y_pos: u8, height: u16) {
        let step = if DOUBLE_HEIGHT { 2 } else { 1 };
        let sprite_width = if DOUBLE_HEIGHT { 15 } else { 7 };
        let sprite_bound = Self::CHIP_SCR_WIDTH as u8 - sprite_width - 1;

        for i in (0..if DOUBLE_HEIGHT { 32 } else { height }).step_by(step) {
            if y_pos == Self::CHIP_SCR_HEIGHT as u8 {
                break;
            }

            let sprite_row = if DOUBLE_HEIGHT {
                let hi = self.ram[((self.i_reg + i) & 0xFFF) as usize] as u64;
                let lo = self.ram[((self.i_reg + i + 1) & 0xFFF) as usize] as u64;
                (hi << 8) | lo
            }
            else {
                self.ram[((self.i_reg + i) & 0xFFF) as usize] as u64
            };

            let sprite_mask = if x_pos > sprite_bound { sprite_row >> (x_pos - sprite_bound) }
            else { sprite_row << (Self::CHIP_SCR_WIDTH as u8 - 1 - x_pos - sprite_width) };

            self.regs[0xF] |= ((self.screen_buf[y_pos as usize] & sprite_mask) != 0) as u8;
            self.screen_buf[y_pos as usize] ^= sprite_mask;
            y_pos += 1;
        }
    }
    fn draw_hires_sprite<const DOUBLE_HEIGHT: bool>(&mut self, x_pos: u8, mut y_pos: u8, height: u16) {
        let step = if DOUBLE_HEIGHT { 2 } else { 1 };
        let sprite_width = if DOUBLE_HEIGHT { 15 } else { 7 };
        let sprite_bound = Self::SCHIP_SCR_WIDTH as u8 - sprite_width - 1;

        for i in (0..if DOUBLE_HEIGHT { 32 } else { height }).step_by(step) {
            if y_pos == Self::SCHIP_SCR_HEIGHT as u8 {
                break;
            }

            let sprite_row = if DOUBLE_HEIGHT {
                let hi = self.ram[((self.i_reg + i) & 0xFFF) as usize] as u64;
                let lo = self.ram[((self.i_reg + i + 1) & 0xFFF) as usize] as u64;
                ((hi << 8) | lo) as u128
            }
            else {
                self.ram[((self.i_reg + i) & 0xFFF) as usize] as u128
            };

            let sprite_mask = if x_pos > sprite_bound { sprite_row >> (x_pos - sprite_bound) }
            else { sprite_row << (Self::SCHIP_SCR_WIDTH as u8 - 1 - x_pos - sprite_width) };

            self.regs[0xF] |= ((self.schip_screen_buf[y_pos as usize] & sprite_mask) != 0) as u8;
            self.schip_screen_buf[y_pos as usize] ^= sprite_mask;
            y_pos += 1;
        }
    }
}