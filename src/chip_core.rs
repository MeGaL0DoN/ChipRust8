use std::path::Path;
use std::{env, fs};
use std::io::ErrorKind;

pub struct ChipCore {
    screen_buf: [u64; ChipCore::SCR_HEIGHT],
    ram: [u8; ChipCore::RAM_SIZE],
    regs: [u8; 16],
    stack: [u16; 16],
    key_state: [bool; 16],
    sp: u16,
    pc: u16,
    i_reg: u16,
    delay_timer: u8,
    sound_timer: u8,
}

impl ChipCore {
    pub const SCR_WIDTH: usize = 64;
    pub const SCR_HEIGHT: usize = 32;
    pub const RAM_SIZE: usize = 4096;

    pub fn new() -> Self {
        ChipCore {
            screen_buf: [0; Self::SCR_HEIGHT],
            ram: [0; Self::RAM_SIZE],
            regs: [0; 16],
            stack: [0; 16],
            key_state: [false; 16],
            sp: 0,
            pc: 0x200,
            i_reg: 0,
            delay_timer: 0,
            sound_timer: 0
        }
    }

    pub fn load_rom(&mut self, path: &Path) -> bool {
        let result = fs::read(path);
        let bytes = if let Ok(val) = result { val } else { return false };

        if bytes.len() <= Self::RAM_SIZE - 0x200 {
            self.ram[0x200..0x200 + bytes.len()].clone_from_slice(&bytes);
            return true
        }

        return false;
    }

    pub fn render_to_rgb_buffer(&mut self, buf: &mut [u32]) {
        for i in 0..Self::SCR_WIDTH * Self::SCR_HEIGHT {
            buf[i] = if ((self.screen_buf[(i >> 6) as usize] >> (63 - (i & 0x3F))) & 0x1) == 1 { 0xFFFFFFFF } else { 0 };
        }
    }

    pub fn execute(&mut self) {
        let opcode: u16 = ((self.ram[(self.pc & 0xFFF) as usize] as u16) << 8) | (self.ram[((self.pc + 1) & 0xFFF) as usize] as u16);
        self.pc += 2;

        let x = || -> usize { ((opcode & 0x0F00) >> 8) as usize };
        let y = || -> usize { ((opcode & 0x00F0) >> 4) as usize };
        let data = || -> u8 { (opcode & 0x00FF) as u8 };
        let addr = || -> u16 { opcode & 0x0FFF };

        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x00E0 => {
                        self.screen_buf.fill(0);
                    }
                    0x00EE => {
                        self.sp = self.sp.wrapping_sub(1) & 0xF;
                        self.pc = self.stack[self.sp as usize];
                    }
                    _ => {
                        println!("Unknown opcode {:X}", opcode);
                    }
                }
            }
            0x1000 => {
                self.pc = addr();
            }
            0x6000 => {
                self.regs[x()] = data();
            }
            0x7000 => {
                self.regs[x()] = self.regs[x()].wrapping_add(data());
            }
            0xA000 => {
                self.i_reg = addr();
            }
            0xD000 => {
                let height = opcode & 0x000F;
                let x_pos = self.regs[x()] % (Self::SCR_WIDTH as u8);
                let mut y_pos = self.regs[y()] % (Self::SCR_HEIGHT as u8);

                self.regs[0xF] = 0;

                for i in 0..height {
                    if y_pos == Self::SCR_HEIGHT as u8 {
                        break;
                    }

                    let sprite_row : u64 = self.ram[((self.i_reg + i) & 0xFFF) as usize] as u64;
                    let sprite_mask: u64 = if x_pos > 56 { sprite_row >> (x_pos - 56) } else { sprite_row << (63 - x_pos - 7) };

                    self.regs[0xF] |= ((self.screen_buf[y_pos as usize] & sprite_mask) != 0) as u8;
                    self.screen_buf[y_pos as usize] ^= sprite_mask;
                    y_pos += 1;
                }
            }
            _ => {
                println!("Unknown opcode {:X}", opcode);
            }
        }
    }
}