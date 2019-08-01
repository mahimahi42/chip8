extern crate clap;
use clap::{Arg, App};

extern crate sdl2;
use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

mod chip8cpu;
use chip8cpu::Cpu;

const SCALE: u32 = 5;
const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;
const SCREEN_WIDTH: u32 = WIDTH * SCALE;
const SCREEN_HEIGHT: u32 = HEIGHT * SCALE;

fn main() {
    let sdl = sdl2::init().unwrap();
    let vid = sdl.video().unwrap();
    let window = vid.window("CHIP-8", SCREEN_WIDTH, SCREEN_HEIGHT)
                    .position_centered()
                    .opengl()
                    .build()
                    .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(pixels::Color::RGB(0,0,0));
    canvas.clear();
    canvas.present();

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
    proc.execute_rom();
}
