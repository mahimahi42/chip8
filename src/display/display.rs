// extern crate sdl2;
// use sdl2::EventPump;
// use sdl2::pixels;
// use sdl2::rect::Rect;
// use sdl2::render::Canvas;
// use sdl2::video::Window;
//
// pub const SCALE: u32 = 10;
// pub const WIDTH: u32 = 64;
// pub const HEIGHT: u32 = 32;
// pub const SCREEN_WIDTH: u32 = WIDTH * SCALE;
// pub const SCREEN_HEIGHT: u32 = HEIGHT * SCALE;
//
// pub struct Display {
//     pub canvas: Canvas<Window>,
//     pub vram: [[u8; WIDTH as usize]; HEIGHT as usize],
//     pub vram_changed: bool,
//     pub event_pump: EventPump,
// }
//
// impl Display {
//     pub fn new(sdl: &sdl2::Sdl) -> Self {
//         let vid = sdl.video().unwrap();
//         let window = vid.window("CHIP-8", SCREEN_WIDTH, SCREEN_HEIGHT)
//                         .position_centered()
//                         .opengl()
//                         .build()
//                         .unwrap();
//
//         let mut canvas = window.into_canvas().build().unwrap();
//
//         canvas.set_draw_color(pixels::Color::RGB(0,0,0));
//         canvas.clear();
//         canvas.present();
//
//         Display {
//             canvas: canvas,
//             vram: [[0; WIDTH as usize]; HEIGHT as usize],
//             vram_changed: false,
//             event_pump: sdl.event_pump().unwrap()
//         }
//     }
//
//     pub fn draw(&mut self) {
//         for (y, row) in self.vram.iter().enumerate() {
//             for (x, &col) in row.iter().enumerate() {
//                 let x = (x as u32) * SCALE;
//                 let y = (y as u32) * SCALE;
//
//                 self.canvas.set_draw_color(Display::pix_color(col));
//                 let _ = self.canvas
//                     .fill_rect(Rect::new(x as i32, y as i32, SCALE, SCALE));
//             }
//         }
//         self.canvas.present();
//     }
//
//     fn pix_color(pix: u8) -> pixels::Color {
//         if pix == 0 {
//             pixels::Color::RGB(0, 0, 0)
//         } else {
//             pixels::Color::RGB(255, 255, 255)
//         }
//     }
// }