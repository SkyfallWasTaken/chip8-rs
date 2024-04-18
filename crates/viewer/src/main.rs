use std::fs;
use std::path::PathBuf;

use color_eyre::{eyre::WrapErr, Result};

use machine::{
    AudioDriver, Drivers, InputDriver, Machine, Quirks,
    CYCLES_PER_SECOND as DEFAULT_CYCLES_PER_SECOND, DISPLAY_HEIGHT, DISPLAY_WIDTH,
};
use macroquad::prelude::*;

use clap::Parser;

#[rustfmt::skip]
const KEY_MAP: [KeyCode; 16] = [
    KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4,
    KeyCode::Q, KeyCode::W, KeyCode::E, KeyCode::R,
    KeyCode::A, KeyCode::S, KeyCode::D, KeyCode::F,
    KeyCode::Z, KeyCode::X, KeyCode::C, KeyCode::V,
];

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the CHIP-8 ROM.
    path: PathBuf,

    /// Logs debugging information after this cycle is executed
    #[arg(long)]
    cycle_to_log: Option<u32>,

    /// Show the current FPS in the top left corner of the screen.
    #[arg(long, short, default_value_t = false)]
    show_fps: bool,

    /// The number of cycles to execute per second.
    #[arg(long, default_value_t = DEFAULT_CYCLES_PER_SECOND)]
    cycles_per_second: usize,
}

#[macroquad::main("CHIP-8 Emulator")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let cli = Cli::parse();

    let rom = fs::read(cli.path).wrap_err("Failed to read ROM")?;

    let mut machine = Machine::from_rom(
        &rom,
        Quirks::modern_chip8(),
        Drivers {
            audio: AudioDriver {
                start_beep: || {
                    log::info!("BEEP");
                },
                stop_beep: || {
                    log::info!("BEEP STOP");
                },
            },
            input: InputDriver {
                get_key_pressed: || {
                    KEY_MAP
                        .iter()
                        .filter_map(|key| {
                            if is_key_down(*key) {
                                Some(match key {
                                    KeyCode::Key1 => 0x1,
                                    KeyCode::Key2 => 0x2,
                                    KeyCode::Key3 => 0x3,
                                    KeyCode::Key4 => 0xC,
                                    KeyCode::Q => 0x4,
                                    KeyCode::W => 0x5,
                                    KeyCode::E => 0x6,
                                    KeyCode::R => 0xD,
                                    KeyCode::A => 0x7,
                                    KeyCode::S => 0x8,
                                    KeyCode::D => 0x9,
                                    KeyCode::F => 0xE,
                                    KeyCode::Z => 0xA,
                                    KeyCode::X => 0x0,
                                    KeyCode::C => 0xB,
                                    KeyCode::V => 0xF,
                                    _ => unreachable!(),
                                })
                            } else {
                                None
                            }
                        })
                        .next()
                },
            },
        },
    );

    let mut current_cycle = 1;
    let mut accumulator = 0.0;
    let cps = cli.cycles_per_second as f32;
    loop {
        machine.decr_timers();
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

        clear_background(BLACK);

        if cli.show_fps {
            draw_text(format!("FPS: {}", get_fps()).as_str(), 0., 16., 32., RED);
        }

        let scale_factor: f32 = f32::min(
            screen_width() / DISPLAY_WIDTH as f32,
            screen_height() / DISPLAY_HEIGHT as f32,
        );
        for ((x, y), pixel_on) in machine.display.indexed_iter() {
            if *pixel_on {
                draw_rectangle(
                    x as f32 * scale_factor,
                    y as f32 * scale_factor,
                    scale_factor,
                    scale_factor,
                    WHITE,
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
