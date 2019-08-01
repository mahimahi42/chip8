use std::fs::File;
use std::io::{Read, Result};
use std::thread;
use std::time::Duration;
use std::fmt::{Display, Formatter, Result as fmtResult};

use rand::Rng;

const RAM: usize = 4096;
const PROG_START: usize = 0x200;

pub struct Opcode {
    opcode: u16,
    nibbles: (u8, u8, u8, u8),
    nnn: usize,
    n: usize,
    x: usize,
    y: usize,
    kk: u8
}

impl Opcode {
    pub fn new(opcode: u16) -> Opcode {
        Opcode {
            opcode: opcode,
            nibbles: (
                ((opcode & 0xF000) >> 12) as u8,
                ((opcode & 0x0F00) >> 8) as u8,
                ((opcode & 0x00F0) >> 4) as u8,
                (opcode & 0x000F) as u8
                ),
            nnn: (opcode & 0xFFF) as usize,
            n: (opcode & 0x000F) as usize,
            x: ((opcode & 0x0F00) >> 8) as usize,
            y: ((opcode & 0x00F0) >> 4) as usize,
            kk: (opcode & 0x00FF) as u8
        }
    }
}

impl Display for Opcode {
    fn fmt(&self, fmt: &mut Formatter) -> fmtResult {
        fmt.write_str(&format!("{:#06X}", self.opcode))?;
        Ok(())
    }
}

pub struct Cpu {
    reg_v: [u8; 16],
    reg_i: usize,
    reg_d: u8,
    reg_s: u8,
    pc: usize,
    sp: usize,
    stack: [usize; 16],
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

    fn opcode(&self) -> Opcode {
        Opcode::new((self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16))
    }

    pub fn execute_rom(&mut self) {
        let tick_delay = Duration::from_millis(200);

        loop {
            let opcode = self.opcode();

            self.execute_opcode(opcode);

            thread::sleep(tick_delay);
        }
    }

    fn execute_opcode(&mut self, opcode: Opcode) {
        match opcode.nibbles.0 {
            0x0 => self.opcode_0x0(opcode),
            0x1 => self.opcode_0x1(opcode),
            0x2 => self.opcode_0x2(opcode),
            0x3 => self.opcode_0x3(opcode),
            0x4 => self.opcode_0x4(opcode),
            0x5 => self.opcode_0x5(opcode),
            0x6 => self.opcode_0x6(opcode),
            0x7 => self.opcode_0x7(opcode),
            //0x8 => self.opcode_0x8(opcode),
            0x9 => self.opcode_0x9(opcode),
            0xA => self.opcode_0xA(opcode),
            0xB => self.opcode_0xB(opcode),
            0xC => self.opcode_0xC(opcode),
            //0xD => self.opcode_0xD(opcode),
            //0xE => self.opcode_0xE(opcode),
            //0xF => self.opcode_0xF(opcode),
            _ => {
                println!("{}", Cpu::unimplemented_opcode(opcode));
                self.pc += 2;
            }
        }
    }

    fn opcode_0x0(&mut self, opcode: Opcode) {
        match opcode.nibbles.2 {
            0xE => match opcode.nibbles.3 {
                0x0 => println!("CLS"),
                0xE => {
                    self.pc = self.stack[self.sp];
                    self.sp -= 1;
                    self.pc += 2;
                },
                _ => {
                    println!("{}", Cpu::unimplemented_opcode(opcode));
                    self.pc += 2;
                }
            }
            _ => {
                println!("{}", Cpu::unimplemented_opcode(opcode));
                self.pc += 2;
            }
        }
    }

    fn opcode_0x1(&mut self, opcode: Opcode) {
        self.pc = opcode.nnn;
        println!("Set the PC!");
    }

    fn opcode_0x2(&mut self, opcode: Opcode) {
        self.sp += 1;
        self.stack[self.sp] = self.pc;
        self.pc = opcode.nnn;
    }

    fn opcode_0x3(&mut self, opcode: Opcode) {
        if self.reg_v[opcode.x] == opcode.kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn opcode_0x4(&mut self, opcode: Opcode) {
        if self.reg_v[opcode.x] != opcode.kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn opcode_0x5(&mut self, opcode: Opcode) {
        if self.reg_v[opcode.x] == self.reg_v[opcode.y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn opcode_0x6(&mut self, opcode: Opcode) {
        self.reg_v[opcode.x] = opcode.kk;
        self.pc += 2;
    }

    fn opcode_0x7(&mut self, opcode: Opcode) {
        self.reg_v[opcode.x] = self.reg_v[opcode.y];
        self.pc += 2;
    }

    fn opcode_0x8(&mut self, opcode: Opcode) {
        
    }

    fn opcode_0x9(&mut self, opcode: Opcode) {
        if self.reg_v[opcode.x] != self.reg_v[opcode.y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    #[allow(non_snake_case)]
    fn opcode_0xA(&mut self, opcode: Opcode) {
        self.reg_i = opcode.nnn;
        self.pc += 2;
    }

    #[allow(non_snake_case)]
    fn opcode_0xB(&mut self, opcode: Opcode) {
        self.pc = opcode.nnn + self.reg_v[0x0] as usize;
    }

    #[allow(non_snake_case)]
    fn opcode_0xC(&mut self, opcode: Opcode) {
        let rnd = rand::thread_rng().gen_range(0, 256) as u8;
        self.reg_v[opcode.x] = rnd & opcode.kk;
    }

    #[allow(non_snake_case)]
    fn opcode_0xD(&mut self, opcode: Opcode) {
        
    }

    #[allow(non_snake_case)]
    fn opcode_0xE(&mut self, opcode: Opcode) {
        
    }

    #[allow(non_snake_case)]
    fn opcode_0xF(&mut self, opcode: Opcode) {
        
    }

    fn unimplemented_opcode(opcode: Opcode) -> String {
        format!("Unimplemented opcode {}", opcode)
    }
}