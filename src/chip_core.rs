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

    fn execute(&mut self) {
        let opcode: u16 = (self.ram[self.pc] << 8) | self.ram[self.pc + 1];
        self.pc += 2;

        match opcode {
            0x00E0 => {
                self.screen_buf.fill(0);
            }
            0x00EE => {
                self.pc = self.stack[self.sp -= 1];
            }
        }
    }
}