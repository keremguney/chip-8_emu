use rand::{Rng, SeedableRng};
use std::fs;

type Chip8Func = fn(&mut Chip8);

const START_ADDR: usize = 0x200;
const MEM_SIZE: usize = 4096;
const FONTSET_SIZE: usize = 80;
const FONTSET_START_ADDR: usize = 0x50;
const VIDEO_WIDTH: usize = 64;
const VIDEO_HEIGHT: usize = 32;

const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    registers: [u8; 16],
    memory: [u8; MEM_SIZE],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    video: [u32; VIDEO_HEIGHT * VIDEO_WIDTH],
    opcode: u16,
    rng: rand::rngs::StdRng,
    table: [Chip8Func; 0xF + 1],
    table0: [Chip8Func; 0xE + 1],
    table8: [Chip8Func; 0xE + 1],
    table_e: [Chip8Func; 0xE + 1],
    table_f: [Chip8Func; 0x65 + 1],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip = Self {
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            pc: START_ADDR as u16,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [0; VIDEO_HEIGHT * VIDEO_WIDTH],
            opcode: 0,
            rng: rand::rngs::StdRng::from_entropy(),
            table: [Self::op_null; 0x10],
            table0: [Self::op_null; 0xF],
            table8: [Self::op_null; 0xF],
            table_e: [Self::op_null; 0xF],
            table_f: [Self::op_null; 0x66],
        };

        chip.table[0x0] = Self::table_0;
        chip.table[0x1] = Self::op_1nnn;
        chip.table[0x2] = Self::op_2nnn;
        chip.table[0x3] = Self::op_3xkk;
        chip.table[0x4] = Self::op_4xkk;
        chip.table[0x5] = Self::op_5xy0;
        chip.table[0x6] = Self::op_6xkk;
        chip.table[0x7] = Self::op_7xkk;
        chip.table[0x8] = Self::table_8;
        chip.table[0x9] = Self::op_9xy0;
        chip.table[0xA] = Self::op_annn;
        chip.table[0xB] = Self::op_bnnn;
        chip.table[0xC] = Self::op_cxkk;
        chip.table[0xD] = Self::op_dxyn;
        chip.table[0xE] = Self::table_e;
        chip.table[0xF] = Self::table_f;

        chip.table0[0x0] = Self::op_00e0;
        chip.table0[0xE] = Self::op_00ee;

        chip.table8[0x0] = Self::op_8xy0;
        chip.table8[0x1] = Self::op_8xy1;
        chip.table8[0x2] = Self::op_8xy2;
        chip.table8[0x3] = Self::op_8xy3;
        chip.table8[0x4] = Self::op_8xy4;
        chip.table8[0x5] = Self::op_8xy5;
        chip.table8[0x6] = Self::op_8xy6;
        chip.table8[0x7] = Self::op_8xy7;
        chip.table8[0xE] = Self::op_8xye;

        chip.table_e[0x1] = Self::op_exa1;
        chip.table_e[0xE] = Self::op_ex9e;

        chip.table_f[0x07] = Self::op_fx07;
        chip.table_f[0x0A] = Self::op_fx0a;
        chip.table_f[0x15] = Self::op_fx15;
        chip.table_f[0x18] = Self::op_fx18;
        chip.table_f[0x1E] = Self::op_fx1e;
        chip.table_f[0x29] = Self::op_fx29;
        chip.table_f[0x33] = Self::op_fx33;
        chip.table_f[0x55] = Self::op_fx55;
        chip.table_f[0x65] = Self::op_fx65;

        for i in 0..FONTSET_SIZE {
            chip.memory[FONTSET_START_ADDR + i] = FONTSET[i];
        }

        chip
    }

    pub fn load_rom(&mut self, filename: &str) -> std::io::Result<()> {
        let contents = fs::read(filename)?;

        if contents.len() > (self.memory.len() - START_ADDR) {
            panic!("ROM is too large for memory!");
        }

        for (i, byte) in contents.into_iter().enumerate() {
            self.memory[START_ADDR + i] = byte;
        }

        Ok(())
    }

    pub fn cycle(&mut self) {
        let pc = self.pc as usize;

        self.opcode = ((self.memory[pc] as u16) << 8) | (self.memory[pc + 1] as u16);
        self.pc += 2;

        let table_index = ((self.opcode & 0xF000) >> 12) as usize;
        (self.table[table_index])(self);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn table_0(&mut self) {
        let index = (self.opcode & 0x000F) as usize;
        (self.table0[index])(self);
    }

    fn table_8(&mut self) {
        let index = (self.opcode & 0x000F) as usize;
        (self.table8[index])(self);
    }

    fn table_e(&mut self) {
        let index = (self.opcode & 0x000F) as usize;
        (self.table_e[index])(self);
    }

    fn table_f(&mut self) {
        let index = (self.opcode & 0x00FF) as usize;
        (self.table_f[index])(self);
    }

    // OPCODES
    fn op_null(&mut self) {}

    fn op_00e0(&mut self) {
        self.video.fill(0);
    }

    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn op_1nnn(&mut self) {
        let addr: u16 = self.opcode & 0xFFF;
        self.pc = addr;
    }

    fn op_2nnn(&mut self) {
        let addr: u16 = self.opcode & 0xFFF;
        self.pc = addr;
    }

    fn op_3xkk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.opcode & 0x00FF) as u8;
    }

    fn op_4xkk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.opcode & 0x00FF) as u8;

        if self.registers[vx as usize] != byte {
            self.pc += 2;
        }
    }

    fn op_5xy0(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

        if self.registers[vx as usize] == self.registers[vy as usize] {
            self.pc += 2;
        }
    }

    fn op_6xkk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.opcode & 0x00FF) as u8;

        self.registers[vx as usize] = byte;
    }

    fn op_7xkk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.opcode & 0x00FF) as u8;

        self.registers[vx as usize] += byte;
    }

    fn op_8xy0(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

        self.registers[vx as usize] = self.registers[vy as usize];
    }

    fn op_8xy1(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

        self.registers[vx as usize] |= self.registers[vy as usize];
    }

    fn op_8xy2(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

        self.registers[vx as usize] &= self.registers[vy as usize];
    }

    fn op_8xy3(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

        self.registers[vx as usize] ^= self.registers[vy as usize];
    }

    fn op_8xy4(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

        let sum: u16 = self.registers[vx as usize] as u16 + self.registers[vy as usize] as u16;

        if sum > 255 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[vx as usize] = (sum & 0xFF) as u8;
    }

    fn op_8xy5(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

        if self.registers[vx as usize] > self.registers[vy as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[vx as usize] =
            self.registers[vx as usize].wrapping_sub(self.registers[vy as usize]);
    }

    fn op_8xy6(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;

        self.registers[0xF] = self.registers[vx as usize] & 0x1;
        self.registers[vx as usize] >>= 1;
    }

    fn op_8xy7(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

        if self.registers[vy as usize] > self.registers[vx as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[vx as usize] = self.registers[vy as usize] - self.registers[vx as usize];
    }

    fn op_8xye(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;

        self.registers[0xF] = (self.registers[vx as usize] & 0x80) >> 7;
        self.registers[vx as usize] <<= 1;
    }

    fn op_9xy0(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

        if self.registers[vx as usize] != self.registers[vy as usize] {
            self.pc += 2;
        }
    }

    fn op_annn(&mut self) {
        let addr: u16 = self.opcode & 0x0FFF;
        self.index = addr;
    }

    fn op_bnnn(&mut self) {
        let addr: u16 = self.opcode & 0x0FFF;

        self.pc = self.registers[0] as u16 + addr;
    }

    fn op_cxkk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.opcode & 0x00FF) as u8;

        self.registers[vx as usize] = self.rng.r#gen::<u8>() & byte;
    }

    fn op_dxyn(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;
        let height: u8 = (self.opcode & 0x000F) as u8;

        let x_pos: u8 = self.registers[vx as usize] % VIDEO_WIDTH as u8;
        let y_pos: u8 = self.registers[vy as usize] % VIDEO_HEIGHT as u8;

        self.registers[0xF] = 0;

        for row in 0..height {
            let sprite_byte: u8 = self.memory[(self.index as u8 + row) as usize];

            for col in 0..8 {
                let sprite_pixel = sprite_byte & (0x80 >> col);

                if sprite_pixel != 0 {
                    let screen_x = x_pos + col;
                    let screen_y = y_pos + row;

                    if screen_x >= VIDEO_WIDTH as u8 || screen_y >= VIDEO_HEIGHT as u8 {
                        continue;
                    }

                    let pixel_index = screen_y * VIDEO_WIDTH as u8 + screen_x;

                    if self.video[pixel_index as usize] == 0xFFFFFFFF {
                        self.registers[0xF] = 1;
                    }

                    self.video[pixel_index as usize] ^= 0xFFFFFFFF;
                }
            }
        }
    }

    fn op_ex9e(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let key: u8 = self.registers[vx as usize];

        if self.keypad[key as usize] != 0 {
            self.pc += 2;
        }
    }

    fn op_exa1(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;

        let key: u8 = self.registers[vx as usize];

        if self.keypad[key as usize] == 0 {
            self.pc += 2;
        }
    }

    fn op_fx07(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;

        self.registers[vx as usize] = self.delay_timer;
    }

    fn op_fx0a(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as u8;

        if let Some(key) = self.keypad.iter().position(|&k| k != 0) {
            self.registers[vx as usize] = key as u8;
        } else {
            self.pc -= 2;
        }
    }

    fn op_fx15(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        self.delay_timer = self.registers[vx as usize];
    }

    fn op_fx18(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        self.sound_timer = self.registers[vx as usize];
    }

    fn op_fx1e(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        self.index += self.registers[vx as usize] as u16;
    }

    fn op_fx29(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let digit: u8 = self.registers[vx as usize];

        self.index = (FONTSET_START_ADDR as u8 + (5 * digit)) as u16;
    }

    fn op_fx33(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let mut value: u8 = self.registers[vx as usize];

        self.memory[(self.index + 2) as usize] = value % 10;
        value /= 10;

        self.memory[(self.index + 1) as usize] = value % 10;
        value /= 10;

        self.memory[self.index as usize] = value % 10;
    }

    fn op_fx55(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;

        for i in 0..=vx {
            self.memory[(self.index + i as u16) as usize] = self.registers[i as usize];
        }
    }

    fn op_fx65(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;

        for i in 0..=vx {
            self.registers[i as usize] = self.memory[(self.index + 1) as usize];
        }
    }
}
