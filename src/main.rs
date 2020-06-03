extern crate clap;
use clap::{Arg, App};

extern crate fps_clock;
use fps_clock::FpsClock;

mod cpu;
use cpu::Chip8Cpu;
//
// mod display;
// use display::Chip8Display;

const FPS: u32 = 60;

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

    let mut proc = Chip8Cpu::new();
    let mut fps_clock = FpsClock::new(FPS);

    proc.load_rom(input_file);

    loop {
        proc.tick();

        fps_clock.tick();
    }
    // proc.execute_rom();
}
