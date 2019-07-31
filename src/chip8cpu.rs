use std::fs::File;
use std::io::{Read, Result};
use std::thread;
use std::time::Duration;

const RAM: usize = 4096;
const PROG_START: usize = 0x200;

pub struct Cpu {
    reg_v: [u8; 16],
    reg_i: u16,
    reg_d: u8,
    reg_s: u8,
    pc: usize,
    sp: usize,
    stack: [u16; 16],
    ram: [u8; RAM],
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            reg_v: [0; 16],
            reg_i: 0,
            reg_d: 0,
            reg_s: 0,
            pc: PROG_START,
            sp: 0,
            stack: [0; 16],
            ram: [0; RAM]
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        if let Ok(rom) = self.read_rom(path) {
            let rom_data: &[u8] = &rom;
            for (i, &data) in rom_data.iter().enumerate() {
                let mem_addr = PROG_START + i;
                if mem_addr < 4096 {
                    self.ram[mem_addr] = data;
                }
            }
        } else {
            println!("COULD NOT LOAD ROM {}", path);
        }
    }

    fn read_rom(&self, path: &str) -> Result<Vec<u8>> {
        let mut file = File::open(path)?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        return Ok(data);
    }

    fn opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    pub fn execute_rom(&mut self) {
        let tick_delay = Duration::from_millis(200);

        loop {
            let opcode = self.opcode();

            self.execute_opcode(opcode);

            thread::sleep(tick_delay);
        }
    }

    fn execute_opcode(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            _ => println!("{}", Cpu::unimplemented_opcode(opcode))
        }
    }

    fn unimplemented_opcode(opcode: u16) -> String {
        format!("Unimplemented opcode {:#06X}", opcode)
    }
}