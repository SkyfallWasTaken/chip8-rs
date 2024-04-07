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

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

pub const PROGRAM_START: u16 = 0x200;

pub const CYCLES_PER_SECOND: usize = 700;

pub struct Quirks {
    pub set_vx_to_vy: bool,
    pub fx_incr_index: bool,
    pub set_vf_on_fx1e_overflow: bool,
    pub bxnn: bool,
}

impl Quirks {
    pub const fn modern_chip8() -> Self {
        Self {
            set_vx_to_vy: true,
            fx_incr_index: false,
            set_vf_on_fx1e_overflow: true,
            bxnn: false,
        }
    }
}

pub struct AudioDriver {
    pub start_beep: fn(),
    pub stop_beep: fn(),
}

pub struct InputDriver {
    pub get_key_pressed: fn() -> Option<u8>,
}

pub struct Drivers {
    pub audio: AudioDriver,
    pub input: InputDriver,
}

impl Drivers {
    pub fn new(audio: AudioDriver, input: InputDriver) -> Self {
        Self { audio, input }
    }

    pub fn noop() -> Self {
        Self {
            audio: AudioDriver {
                start_beep: || {},
                stop_beep: || {},
            },
            input: InputDriver {
                get_key_pressed: || None,
            },
        }
    }
}

#[must_use]
pub struct Machine {
    pub memory: [u8; 4096],
    pub display: Array2<bool>,
    pub pc: u16,
    pub index: u16,
    pub stack: Vec<u16>,

    pub dt: u8,
    pub st: u8,
    pub registers: [u8; 16],

    pub is_dirty: bool,

    pub quirks: Quirks,
    pub drivers: Drivers,
}

impl Machine {
    pub fn from_rom(rom: &[u8], quirks: Quirks, drivers: Drivers) -> Self {
        let mut memory = [0; 4096];
        memory[FONT_START as usize..FONT_START as usize + FONT.len()].copy_from_slice(&FONT);
        memory[PROGRAM_START as usize..PROGRAM_START as usize + rom.len()].copy_from_slice(rom);

        Self {
            memory,
            display: Array2::from_elem([DISPLAY_WIDTH, DISPLAY_HEIGHT], false),
            pc: PROGRAM_START,
            index: 0,
            stack: Vec::new(),

            dt: 0,
            st: 0,
            registers: [0; 16],

            is_dirty: false,

            quirks,
            drivers,
        }
    }

    pub fn decr_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            self.st -= 1;
            if self.st == 0 {
                (self.drivers.audio.stop_beep)();
            }
        }
    }

    pub fn cycle(&mut self) {
        let instr = ((self.memory[self.pc as usize] as u16) << 8)
            | self.memory[self.pc as usize + 1] as u16;
        self.pc += 2;

        if self.st > 0 {
            todo!("Start a beep")
        }

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
                // Clear the display
                self.display.fill(false);
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
                self.registers[x] = self.registers[x].wrapping_add(nn as u8);
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

                self.registers[x] = result;

                if did_overflow {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
            }
            (0x08, _, _, 0x05) => {
                // Set register `x` to `x` - `y`
                let original_x = self.registers[x];
                self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);

                if original_x >= self.registers[y] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
            }
            (0x08, _, _, 0x07) => {
                // Set register `x` to `y` - `x`
                let original_y = self.registers[y];
                self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);

                if original_y >= self.registers[x] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
            }
            (0x08, _, _, 0x06) => {
                // Shift the value of `x` one bit to the right (8XY6)
                if self.quirks.set_vx_to_vy {
                    self.registers[x] = self.registers[y];
                }

                let original_x = self.registers[x];
                self.registers[x] >>= 1;
                self.registers[0xF] = original_x & 0x1;
            }
            (0x08, _, _, 0x0E) => {
                // Shift the value of `x` one bit to the left (8XY6)
                if self.quirks.set_vx_to_vy {
                    self.registers[x] = self.registers[y];
                }

                let original_x = self.registers[x];
                self.registers[x] <<= 1;
                self.registers[0xF] = (original_x & 0x80) >> 7;
            }

            (0x0D, _, _, _) => {
                // Draw sprite at `x`, `y` with height `n` (DXYN)
                let mut x_coord = self.registers[x] as usize % DISPLAY_WIDTH;
                let mut y_coord = self.registers[y] as usize % DISPLAY_HEIGHT;

                let initial_x = x_coord;

                self.registers[0xF] = 0;

                self.is_dirty = true;

                for yline in 0..n {
                    let sprite_data = self.memory[self.index as usize + yline as usize];

                    for bit in get_bits(sprite_data) {
                        if bit && self.display[(x_coord, y_coord)] {
                            self.display[(x_coord, y_coord)] = false;
                            self.registers[0xF] = 1;
                        } else if bit {
                            self.display[(x_coord, y_coord)] = true;
                        }

                        // If you reach the right edge of the display, stop drawing this row
                        x_coord += 1;
                        if x_coord == DISPLAY_WIDTH {
                            break;
                        }
                    }

                    x_coord = initial_x;

                    y_coord += 1;
                    if y_coord == DISPLAY_HEIGHT {
                        break;
                    }
                }
            }

            (0x0F, _, 0x05, _) => {
                // For FX55, the value of each variable register from V0 to VX inclusive
                // (if X is 0, then only V0) will be stored in successive memory addresses,
                // starting with the one that’s stored in I. V0 will be stored at the address
                // in I, V1 will be stored in I + 1, and so on, until VX is stored in I + X.
                for x in 0..=x {
                    self.memory[self.index as usize + x] = self.registers[x];
                    if self.quirks.fx_incr_index {
                        self.index += 1
                    }
                }
            }
            (0x0F, _, 0x06, _) => {
                // FX65 does the opposite; it takes the value stored at the
                // memory addresses and loads them into the variable registers instead.
                for x in 0..=x {
                    self.registers[x] = self.memory[self.index as usize + x];
                    if self.quirks.fx_incr_index {
                        self.index += 1
                    }
                }
            }

            (0x0F, _, 0x01, 0x0E) => {
                // The index register I will get the value in VX added to it.
                let result = self.index.wrapping_add(self.registers[x] as u16);
                self.index = result;
                if (result <= 0x0FFF || result >= 0x1000) && self.quirks.set_vf_on_fx1e_overflow {
                    self.registers[0xF] = 1;
                }
            }

            (0x0B, _, _, _) => {
                if self.quirks.bxnn {
                    // Jump to the address XNN, plus the value in the register VX.
                    self.pc = nnn + self.registers[x] as u16;
                } else {
                    self.pc = nnn + self.registers[0] as u16;
                }
            }

            (0x0F, _, 0x03, 0x03) => {
                // Takes the number in VX (which is one byte, so it can be any number from 0 to 255) and
                // converts it to three decimal digits, storing these digits in memory at
                // the address in the index register I. For example, if VX contains 156 (or 9C in hexadecimal),
                // it would put the number 1 at the address in I, 5 in address I + 1, and 6 in address I + 2.
                let value = self.registers[x];
                self.memory[self.index as usize] = value / 100;
                self.memory[self.index as usize + 1] = (value / 10) % 10;
                self.memory[self.index as usize + 2] = value % 10;
            }

            // Input
            (0x0F, _, 0x00, 0x0A) => match (self.drivers.input.get_key_pressed)() {
                // This instruction “blocks”; it stops executing instructions and waits for
                // key input (or loops forever, unless a key is pressed).
                // To loop while still decrementing the times, we just decrement the program counter.
                // This means that the program will go to this instruction again and again, until
                // a key is pressed.
                Some(key) => {
                    log::debug!("Key pressed: {:X}", key);
                    self.registers[x] = key;
                }
                None => self.pc -= 2,
            },
            (0x0E, _, 0x09, 0x0E) => {
                // Skip next instruction if key with the value of VX is pressed
                if (self.drivers.input.get_key_pressed)() == Some(self.registers[x]) {
                    self.pc += 2;
                }
            }
            (0x0E, _, 0x0A, 0x01) => {
                // Skip next instruction if key with the value of VX is not pressed
                if (self.drivers.input.get_key_pressed)() != Some(self.registers[x]) {
                    self.pc += 2;
                }
            }

            // Timers
            (0x0F, _, 0x00, 0x07) => {
                // Set VX to the value of the delay timer
                self.registers[x] = self.dt;
            }
            (0x0F, _, 0x01, 0x05) => {
                // Sets the delay timer to the value in VX
                self.dt = self.registers[x];
            }
            (0x0F, _, 0x01, 0x08) => {
                // Sets the sound timer to the value in VX
                self.st = self.registers[x];
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
