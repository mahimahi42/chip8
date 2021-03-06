use sdl2::audio::{AudioDevice, AudioCallback, AudioSpecDesired};

pub struct Chip8Audio {
    device: AudioDevice<SquareWave>
}

impl Chip8Audio {
    pub fn new(sdl: &sdl2::Sdl) -> Self {
        let audio_subsystem = sdl.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            println!("{:?}", spec);

            SquareWave {
                phase_inc: 240.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        }).unwrap();

        Chip8Audio { device: device }
    }

    pub fn play(&self) { self.device.resume(); }
    pub fn stop(&self) { self.device.pause(); }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = self.volume * if self.phase < 0.5 { 1.0 } else { -1.0 };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
