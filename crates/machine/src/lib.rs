use log::{error, warn};
use ndarray::Array2;

pub const FONT: [u8; 80] = [
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
pub const FONT_START: u16 = 0x050;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub const PROGRAM_START: u16 = 0x200;

pub struct Machine {
    pub memory: [u8; 4096],
    pub screen: Array2<bool>,
    pub pc: u16,
    pub index: u16,
    pub stack: Vec<u16>,

    pub dt: u8,
    pub st: u8,
    pub registers: [u8; 16],

    pub is_dirty: bool,
}

impl Machine {
    pub fn from_rom(rom: &[u8]) -> Self {
        let mut memory = [0; 4096];
        memory[FONT_START as usize..FONT_START as usize + FONT.len()].copy_from_slice(&FONT);
        memory[PROGRAM_START as usize..PROGRAM_START as usize + rom.len()].copy_from_slice(&rom);

        Self {
            memory,
            screen: Array2::from_elem([SCREEN_WIDTH, SCREEN_HEIGHT], false),
            pc: PROGRAM_START,
            index: 0,
            stack: Vec::new(),

            dt: 0,
            st: 0,
            registers: [0; 16],

            is_dirty: false,
        }
    }

    pub fn cycle(&mut self) {
        let instr = ((self.memory[self.pc as usize] as u16) << 8)
            | self.memory[self.pc as usize + 1] as u16;
        self.pc += 2;

        let first_nibble = (instr >> 12) & 0xF;
        let second_nibble = (instr >> 8) & 0xF;
        let third_nibble = (instr >> 4) & 0xF;
        let fourth_nibble = instr & 0xF;

        let x = second_nibble as usize;
        let y = third_nibble as usize;
        let n = fourth_nibble;
        let nn = (third_nibble << 4) | fourth_nibble;
        let nnn = (second_nibble << 8) | (third_nibble << 4) | fourth_nibble;

        match (first_nibble, second_nibble, third_nibble, fourth_nibble) {
            (0x00, _, _, 0x00) => {
                // Clear the screen
                self.screen.fill(false);
                self.is_dirty = true;
            }

            (0x01, _, _, _) => {
                // Jump to address nnn
                self.pc = nnn;
            }

            (0x02, _, _, _) => {
                // Call subroutine at nnn
                self.stack.push(self.pc);
                self.pc = nnn;
            }
            (0x00, _, _, 0x0E) => {
                // Returning from a subroutine
                match self.stack.pop() {
                    Some(addr) => self.pc = addr,
                    None => warn!("Attempted to return from a subroutine with an empty stack"),
                }
            }

            (0x03, _, _, _) => {
                // Skip next instruction if register `x` equals `nn`
                if self.registers[x] == nn as u8 {
                    self.pc += 2;
                }
            }
            (0x04, _, _, _) => {
                // Skip next instruction if register `x` doesn't equal `nn`
                if self.registers[x] != nn as u8 {
                    self.pc += 2;
                }
            }
            (0x05, _, _, _) => {
                // Skip next instruction if register `x` equals register `y`
                if self.registers[x] == self.registers[y] {
                    self.pc += 2;
                }
            }
            (0x09, _, _, _) => {
                // Skip next instruction if register `x` doesn't equal register `y`
                if self.registers[x] != self.registers[y] {
                    self.pc += 2;
                }
            }

            (0x06, _, _, _) => {
                // Set register `x` to `nn`
                self.registers[x] = nn as u8;
            }
            (0x07, _, _, _) => {
                // Add `nn` to register `x`
                self.registers[x] = self.registers[x].overflowing_add(nn as u8).0;
            }
            (0x0A, _, _, _) => {
                // Set index register to `nnn`
                self.index = nnn;
            }

            (0x08, _, _, 0x00) => {
                // Set register `x` to the value of register `y`
                self.registers[x] = self.registers[y];
            }
            (0x08, _, _, 0x01) => {
                // Set register `x` to `x` OR `y`
                self.registers[x] |= self.registers[y];
            }
            (0x08, _, _, 0x02) => {
                // Set register `x` to `x` AND `y`
                self.registers[x] &= self.registers[y];
            }
            (0x08, _, _, 0x03) => {
                // Set register `x` to `x` XOR `y`
                self.registers[x] ^= self.registers[y];
            }
            (0x08, _, _, 0x04) => {
                // Add register `y` to register `x`
                // Set register `F` to 1 if there's an overflow, 0 otherwise
                let (result, did_overflow) = self.registers[x].overflowing_add(self.registers[y]);

                if did_overflow {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }

                self.registers[x] = result;
            }
            (0x08, _, _, 0x05) => {
                // Set register `x` to `x` - `y`
                let (result, did_overflow) = self.registers[x].overflowing_sub(self.registers[y]);

                if did_overflow {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }

                self.registers[x] = result;
            }
            (0x08, _, _, 0x07) => {
                // Set register `x` to `y` - `x`
                let (result, did_overflow) = self.registers[y].overflowing_sub(self.registers[x]);

                if did_overflow {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }

                self.registers[x] = result;
            }
            /* (0x08, _, _, 0x06) => {}
            (0x08, _, _, 0x0E) => {} */
            (0x0D, _, _, _) => {
                // Draw sprite at `x`, `y` with height `n` (DXYN)
                let mut x_coord = self.registers[x] as usize % SCREEN_WIDTH;
                let mut y_coord = self.registers[y] as usize % SCREEN_HEIGHT;

                let initial_x = x_coord;

                self.registers[0x0F] = 0;

                self.is_dirty = true;

                for yline in 0..n {
                    let sprite_data = self.memory[self.index as usize + yline as usize];

                    for bit in get_bits(sprite_data) {
                        if bit && self.screen[(x_coord, y_coord)] {
                            self.screen[(x_coord, y_coord)] = false;
                            self.registers[0x0F] = 1;
                        } else if bit {
                            self.screen[(x_coord, y_coord)] = true;
                        }

                        // If you reach the right edge of the screen, stop drawing this row
                        x_coord += 1;
                        if x_coord == SCREEN_WIDTH - 1 {
                            break;
                        }
                    }

                    x_coord = initial_x;

                    y_coord += 1;
                    if y_coord == SCREEN_HEIGHT {
                        break;
                    }
                }
            }
            _ => {
                error!("Unknown instruction: {:04X}", instr);
            }
        }
    }
}

/// Get the bits of a byte in **big-endian** order.
fn get_bits(value: u8) -> [bool; 8] {
    let mut bits = [false; 8];
    for i in 0..8 {
        bits[7 - i] = (value & (1 << i)) != 0;
    }

    bits
}
