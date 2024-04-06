use std::fs;
use std::path::PathBuf;

use color_eyre::{eyre::WrapErr, Result};

use machine::{Machine, Quirks, CYCLES_PER_SECOND};
use macroquad::prelude::*;

const SCALE_FACTOR: i32 = 10;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the CHIP-8 ROM.
    path: PathBuf,

    /// Logs debugging information after this cycle is executed
    #[arg(long, short)]
    cycle_to_log: Option<u32>,
}

#[macroquad::main("CHIP-8 Emulator")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let cli = Cli::parse();

    let rom = fs::read(cli.path).wrap_err("Failed to read ROM")?;
    let mut machine = Machine::from_rom(&rom, Quirks::modern());

    let mut current_cycle = 1;
    let mut accumulator = 0.0;
    let cps = CYCLES_PER_SECOND as f32;
    loop {
        accumulator += get_frame_time();
        while accumulator >= 1.0 / cps {
            machine.cycle();
            accumulator -= 1.0 / cps;
        }
        current_cycle += 1;

        if is_key_released(KeyCode::Right) {
            log_debug_info(&machine, current_cycle);
        }

        if let Some(cycle_to_log) = cli.cycle_to_log {
            if cycle_to_log == current_cycle {
                log_debug_info(&machine, current_cycle);
            }
        }

        clear_background(WHITE);

        draw_text(format!("FPS: {}", get_fps()).as_str(), 0., 16., 32., RED);

        for ((x, y), pixel_on) in machine.screen.indexed_iter() {
            if *pixel_on {
                draw_rectangle(
                    x as f32 * SCALE_FACTOR as f32,
                    y as f32 * SCALE_FACTOR as f32,
                    SCALE_FACTOR as f32,
                    SCALE_FACTOR as f32,
                    BLACK,
                );
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
