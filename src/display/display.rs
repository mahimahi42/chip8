extern crate sdl2;
use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const SCALE: u32 = 10;
const HEIGHT: u32 = 32;
const WIDTH: u32 = 64;
const GAME_SCREEN_HEIGHT: u32 = SCALE * HEIGHT;
const GAME_SCREEN_WIDTH: u32 = SCALE * WIDTH;
const SCREEN_HEIGHT: u32 = GAME_SCREEN_HEIGHT + 100;
const SCREEN_WIDTH: u32 = GAME_SCREEN_WIDTH + 200;

pub struct Chip8Display {
    canvas: Canvas<Window>,
}

impl Chip8Display {
    pub fn new(sdl: &sdl2::Sdl, debug: bool) -> Self {
        let vid = sdl.video().unwrap();
        let window = if debug {
            vid.window("CHIP-8", SCREEN_WIDTH, SCREEN_HEIGHT)
                        .position_centered()
                        .opengl()
                        .build()
                        .unwrap()
        } else {
            vid.window("CHIP-8", GAME_SCREEN_WIDTH, GAME_SCREEN_HEIGHT)
                        .position_centered()
                        .opengl()
                        .build()
                        .unwrap()
        };


        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Chip8Display {
            canvas: canvas,
        }
    }

    pub fn draw(&mut self, vram: &[[u8; WIDTH as usize]; HEIGHT as usize]) {
        for (y, row) in vram.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                let x = (x as u32) * SCALE;
                let y = (y as u32) * SCALE;

                self.canvas.set_draw_color(Chip8Display::pix_color(col));
                self.canvas.fill_rect(Rect::new(x as i32, y as i32, SCALE, SCALE)).unwrap();
            }
        }
        self.canvas.present();
    }

    fn pix_color(pix: u8) -> pixels::Color {
        if pix == 0 {
            pixels::Color::RGB(0, 0, 0)
        } else {
            pixels::Color::RGB(255, 255, 255)
        }
    }
}
