use std::fs;

use color_eyre::Result;

mod machine;
use machine::Machine;

fn main() -> Result<()> {
    color_eyre::install()?;

    let data = fs::read("rom.ch8")?;
    let bytes: &[u8] = &data;

    let mut machine = Machine::from_rom(bytes);

    loop {
        machine.cycle();
    }
}
