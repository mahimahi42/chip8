extern crate clap;
use clap::{Arg, App};

mod chip8cpu;
use chip8cpu::Cpu;

fn main() {
    let args = App::new("CHIP-8 Emulator")
                    .version("1.0")
                    .author("Bryce Davis <me@bryceadavis.com>")
                    .about("CHIP-8 Emulator written in Rust")
                        .arg(Arg::with_name("input_file")
                            .help("Input ROM file")
                            .required(true)
                            .index(1))
                        .get_matches();

    let input_file = args.value_of("input_file").unwrap();

    let mut proc: Cpu = Cpu::new();

    proc.load_rom(input_file);

    println!("{:#06X} {:#06X}", proc.pc, proc.opcode());
}
