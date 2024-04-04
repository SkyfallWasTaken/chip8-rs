use std::env;
use std::fs;

use machine::{Machine, Quirks, SCREEN_HEIGHT, SCREEN_WIDTH};
use macroquad::prelude::*;

const SCALE_FACTOR: i32 = 10;

#[macroquad::main("CHIP-8 Emulator")]
async fn main() {
    env_logger::init();

    let path = env::args().nth(1).expect("No path specified");
    let cycle_to_log = env::args().nth(2).map(|n| n.parse::<u32>().unwrap());

    let rom = fs::read(path).unwrap();
    let mut machine = Machine::from_rom(&rom, Quirks::modern());

    let mut current_cycle = 1;
    loop {
        machine.cycle();
        current_cycle += 1;

        if is_key_down(KeyCode::Right) {
            log_debug_info(&machine, current_cycle);
        }

        if let Some(cycle_to_log) = cycle_to_log {
            if cycle_to_log == current_cycle {
                log_debug_info(&machine, current_cycle);
            }
        }

        clear_background(WHITE);

        draw_text(format!("FPS: {}", get_fps()).as_str(), 0., 16., 32., RED);

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

fn log_debug_info(machine: &Machine, cycle_count: u32) {
    println!("=====BEGIN DEBUG INFO FOR CYCLE {cycle_count}=====");
    dbg!(&machine.registers);
    dbg!(&machine.pc);
    dbg!(&machine.index);
    dbg!(&machine.st);
    dbg!(&machine.stack);
    dbg!(&machine.dt);
    println!("======END DEBUG INFO FOR CYCLE {cycle_count}======")
}
