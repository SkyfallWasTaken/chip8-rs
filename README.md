# CHIP-8

This is a basic CHIP-8 emulator, written in Rust. It uses [Macroquad](https://macroquad.rs) for the viewer, and is platform-agnostic (but currently only has one user - the viewer).

![image](https://github.com/SkyfallWasTaken/chip8-rs/assets/55807755/942f0bea-c042-4ce3-85df-10b21a89340b)

## Running the emulator

**Usage:** viewer [OPTIONS] \<PATH\>

**Arguments:**

- \<PATH\>  The path to the CHIP-8 ROM

**Options:**

- **--cycle-to-log** <CYCLE_TO_LOG>
  
  Logs debugging information after this cycle is executed
- **-s, --show-fps**
  
  Show the current FPS in the top left corner of the screen
- **--cycles-per-second** <CYCLES_PER_SECOND>
  
  The number of cycles to execute per second [default: 700]
- -h, --help
  
  Print help
- -V, --version

  Print version
