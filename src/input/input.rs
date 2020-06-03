use sdl2::{keyboard::Keycode, EventPump};

pub struct Chip8Input {
    pub event_pump: EventPump
}

impl Chip8Input {
    pub fn new(sdl: &sdl2::Sdl) -> Self {
        Chip8Input {
            event_pump: sdl.event_pump().unwrap()
        }
    }

    pub fn poll(&self) -> [bool; 16] {
        let keyboard: Vec<Keycode> = self.event_pump.keyboard_state()
                                      .pressed_scancodes()
                                      .filter_map(Keycode::from_scancode)
                                      .collect();
        let mut keys = [false; 16];

        for key in keyboard {
            let index = match key {
                Keycode::Num1 => Some(0x1),
                Keycode::Num2 => Some(0x2),
                Keycode::Num3 => Some(0x3),
                Keycode::Num4 => Some(0xc),
                Keycode::Q => Some(0x4),
                Keycode::W => Some(0x5),
                Keycode::E => Some(0x6),
                Keycode::R => Some(0xd),
                Keycode::A => Some(0x7),
                Keycode::S => Some(0x8),
                Keycode::D => Some(0x9),
                Keycode::F => Some(0xe),
                Keycode::Z => Some(0xa),
                Keycode::X => Some(0x0),
                Keycode::C => Some(0xb),
                Keycode::V => Some(0xf),
                _ => None,
            };

            if let Some(i) = index {
                keys[i] = true;
            }
        }

        keys
    }
}
