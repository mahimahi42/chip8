extern crate clap;
use clap::{Arg, App};

// mod cpu;
// use cpu::Cpu;
//
// mod display;
// use display::Chip8Display;

fn main() {
    // let sdl = sdl2::init().unwrap();
    // let mut display = Chip8Display::new(&sdl);

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

    // let mut proc: Cpu = Cpu::new(&mut display);
    //
    // proc.load_rom(input_file);
    // proc.execute_rom();
}
