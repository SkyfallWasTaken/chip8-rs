use std::fs;

use machine::{Machine, SCREEN_HEIGHT, SCREEN_WIDTH};
use macroquad::{experimental::coroutines::wait_seconds, prelude::*};

const SCALE_FACTOR: i32 = 20;

type Point = (i16, i16);

#[macroquad::main("CHIP-8 Emulator")]
async fn main() {
    let rom = fs::read("rom.ch8").unwrap();
    let mut machine = Machine::from_rom(&rom);

    loop {
        machine.cycle();

        clear_background(WHITE);

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                if machine.screen[(x, y)] == true {
                    draw_rectangle(
                        x as f32 * SCALE_FACTOR as f32,
                        y as f32 * SCALE_FACTOR as f32,
                        SCALE_FACTOR as f32,
                        SCALE_FACTOR as f32,
                        BLACK,
                    );
                }
            }
        }

        next_frame().await;
    }
}
