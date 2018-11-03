mod cpu;

use cpu::Cpu;

fn main() {
    let mut computer = Cpu::new();
    computer.load_rom();

    computer.cycle();
}
