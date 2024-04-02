use std::env;
use std::fs;

use machine::{Machine, SCREEN_HEIGHT, SCREEN_WIDTH};
use macroquad::prelude::*;

const SCALE_FACTOR: i32 = 10;

#[macroquad::main("CHIP-8 Emulator")]
async fn main() {
    let path = env::args().nth(1).expect("No path specified");
    let rom = fs::read(path).unwrap();
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
