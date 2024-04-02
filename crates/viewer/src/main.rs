use std::fs;

use machine::{Machine, SCREEN_HEIGHT, SCREEN_WIDTH};
use macroquad::{experimental::coroutines::wait_seconds, prelude::*};

const SCALE_FACTOR: i32 = 20;

type Point = (i16, i16);

#[macroquad::main("CHIP-8 Emulator")]
async fn main() {
    let rom = fs::read("rom.ch8").unwrap();
    let mut machine = Machine::from_rom(&rom);

    for i in 0..40 {
        machine.cycle();
        if i == 39 {
            println!("39!");
        }
    }
}
