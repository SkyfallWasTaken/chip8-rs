use machine::Machine;

const CYCLE_NUM: usize = 20;

#[test]
fn ibm_rom() {
    let rom = include_bytes!("../../../roms/ibm-logo.ch8");
    let mut machine = Machine::from_rom(rom);

    for _ in 0..CYCLE_NUM + 1 {
        machine.cycle();
    }

    assert_eq!(
        machine.registers,
        [49, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    // TODO: test screen, PC, index, stack, dt, st
}
