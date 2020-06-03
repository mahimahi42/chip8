extern crate clap;
use clap::{Arg, App};

extern crate fps_clock;
use fps_clock::FpsClock;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod cpu;
use cpu::Chip8Cpu;
mod display;
use display::Chip8Display;
mod input;
use input::Chip8Input;
mod audio;
use audio::Chip8Audio;

const FPS: u32 = 60;

fn main() {
    let sdl = sdl2::init().unwrap();

    let args = App::new("CHIP-8 Emulator")
                    .version("1.0")
                    .author("Bryce Davis <me@bryceadavis.com>")
                    .about("CHIP-8 Emulator written in Rust")
                    .arg(Arg::with_name("debug")
                        .short("d")
                        .long("debug")
                        .help("Enable debugging mode")
                        .takes_value(false))
                    .arg(Arg::with_name("input_file")
                        .help("Input ROM file")
                        .required(true)
                        .index(1))
                    .get_matches();

    let input_file = args.value_of("input_file").unwrap();
    let debug = args.is_present("debug");

    let mut proc = Chip8Cpu::new();
    let mut display = Chip8Display::new(&sdl, debug);
    let mut input = Chip8Input::new(&sdl);
    let audio = Chip8Audio::new(&sdl);
    let mut fps_clock = FpsClock::new(FPS);

    proc.load_rom(input_file);

    'game_loop:loop {
        for event in input.event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape), ..
                } => { break 'game_loop },
                _ => {}
            }
        }

        proc.tick(input.poll());

        if proc.vram_update {
            display.draw(&proc.vram);
        }

        if proc.beep {
            audio.play();
        } else {
            audio.stop();
        }

        fps_clock.tick();
    }
}
