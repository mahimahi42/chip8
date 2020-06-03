use std::fs::File;
use std::io::{Read, Result};
use std::fmt::{Display, Formatter, Result as fmtResult};

extern crate rand;
use rand::RngCore;
extern crate rand_pcg;

const RAM: usize = 4096;
const HEIGHT: usize = 32;
const WIDTH: usize = 64;
const PROG_START: usize = 0x200;
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80,
];

#[derive(Debug)]
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
    pub fn new(opcode: u16) -> Self {
        Opcode {
            opcode,
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

impl PartialEq for Opcode {
    fn eq(&self, rhs: &Opcode) -> bool {
        if self.opcode != rhs.opcode ||
           self.nibbles != rhs.nibbles ||
           self.nnn != rhs.nnn ||
           self.n != rhs.n ||
           self.x != rhs.x ||
           self.y != rhs.y ||
           self.kk != rhs.kk { false }
        else { true }
    }
}

impl From<u16> for Opcode {
    fn from(op: u16) -> Self {
        Opcode::new(op)
    }
}

pub struct Chip8Cpu {
    reg_v: [u8; 16],
    reg_i: usize,
    reg_d: u8,
    reg_s: u8,
    pc: usize,
    sp: usize,
    stack: [usize; 16],
    ram: [u8; RAM],
    pub vram: [[u8; WIDTH]; HEIGHT],
    pub vram_update: bool,
    pub beep: bool,
}

impl Chip8Cpu {
    pub fn new() -> Self {
        let mut ram = [0; RAM];
        for i in 0..FONT.len() {
            ram[i] = FONT[i];
        }

        Chip8Cpu {
            reg_v: [0; 16],
            reg_i: 0,
            reg_d: 0,
            reg_s: 0,
            pc: PROG_START,
            sp: 0,
            stack: [0; 16],
            ram,
            vram: [[0; WIDTH]; HEIGHT],
            vram_update: false,
            beep: false
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

    fn fetch_opcode(&self) -> Opcode {
        Opcode::new((self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16))
    }

    pub fn tick(&mut self, input: [bool; 16]) {
        self.decode_opcode(self.fetch_opcode(), input);

        if self.reg_d > 0 { self.reg_d -= 1; }
        if self.reg_s > 0 {
            self.reg_s -= 1;
            self.beep = true;
        } else {
            self.beep = false;
        }
    }

    fn decode_opcode(&mut self, op: Opcode, input: [bool; 16]) {
        println!("Opcode: {}", op);
        match op.nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.cls_00e0(),
            (0x0, 0x0, 0xE, 0xE) => self.ret_00ee(),
            (0x1, _, _, _) => self.jp_addr_1nnn(op),
            (0x2, _, _, _) => self.call_addr_2nnn(op),
            (0x3, _, _, _) => self.se_vx_kk_3xkk(op),
            (0x4, _, _, _) => self.sne_vx_kk_4xkk(op),
            (0x5, _, _, 0x0) => self.se_vx_vy_5xy0(op),
            (0x6, _, _, _) => self.ld_vx_kk_6xkk(op),
            (0x7, _, _, _) => self.add_vx_kk_7xkk(op),
            (0x8, _, _, 0x0) => self.ld_vx_vy_8xy0(op),
            (0x8, _, _, 0x1) => self.or_vx_vy_8xy1(op),
            (0x8, _, _, 0x2) => self.and_vx_vy_8xy2(op),
            (0x8, _, _, 0x3) => self.xor_vx_vy_8xy3(op),
            (0x8, _, _, 0x4) => self.add_vx_vy_8xy4(op),
            (0x8, _, _, 0x5) => self.sub_vx_vy_8xy5(op),
            (0x8, _, _, 0x6) => self.shr_vx_8xy6(op),
            (0x8, _, _, 0x7) => self.subn_vx_vy_8xy7(op),
            (0x8, _, _, 0xE) => self.shl_vx_8xye(op),
            (0x9, _, _, 0x0) => self.sne_vx_vy_9xy0(op),
            (0xA, _, _, _) => self.ld_i_addr_annn(op),
            (0xB, _, _, _) => self.jp_v0_addr_bnnn(op),
            (0xC, _, _, _) => self.rnd_vx_kk_cxkk(op),
            (0xD, _, _, _) => self.drw_vx_vy_n_dxyn(op),
            (0xE, _, 0x9, 0xE) => self.skp_vx_ex9e(op, input),
            (0xE, _, 0xA, 0x1) => self.sknp_vx_exa1(op, input),
            (0xF, _, 0x0, 0x7) => self.ld_vx_dt_fx07(op),
            (0xF, _, 0x0, 0xA) => self.ld_vx_k_fx0a(op, input),
            (0xF, _, 0x1, 0x5) => self.ld_dt_vx_fx15(op),
            (0xF, _, 0x1, 0x8) => self.ld_st_vx_fx18(op),
            (0xF, _, 0x1, 0xE) => self.add_i_vx_fx1e(op),
            (0xF, _, 0x2, 0x9) => self.ld_f_vx_fx29(op),
            (0xF, _, 0x3, 0x3) => self.ld_b_vx_fx33(op),
            (0xF, _, 0x5, 0x5) => self.ld_i_vx_fx55(op),
            (0xF, _, 0x6, 0x5) => self.ld_vx_i_fx65(op),
            _ => {
                println!("Unimplemented opcode: {}", op);
                self.pc += 2;
            }
        }
    }

    fn cls_00e0(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.vram[y][x] = 0x0;
            }
        }
        self.vram_update = true;
        self.pc += 2;
    }

    fn ret_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp];
    }

    fn jp_addr_1nnn(&mut self, op: Opcode) {
        self.pc = op.nnn;
    }

    fn call_addr_2nnn(&mut self, op: Opcode) {
        self.stack[self.sp] = self.pc + 2;
        self.sp += 1;
        self.pc = op.nnn;
    }

    fn se_vx_kk_3xkk(&mut self, op: Opcode) {
        self.pc += if self.reg_v[op.x] == op.kk { 4 } else { 2 };
    }

    fn sne_vx_kk_4xkk(&mut self, op: Opcode) {
        self.pc += if self.reg_v[op.x] != op.kk { 4 } else { 2 };
    }

    fn se_vx_vy_5xy0(&mut self, op: Opcode) {
        self.pc += if self.reg_v[op.x] == self.reg_v[op.y] { 4 } else { 2 };
    }

    fn ld_vx_kk_6xkk(&mut self, op: Opcode) {
        self.reg_v[op.x] = op.kk;
        self.pc += 2;
    }

    fn add_vx_kk_7xkk(&mut self, op: Opcode) {
        let tmp = self.reg_v[op.x] as u16 + op.kk as u16;
        self.reg_v[op.x] = tmp as u8;
        self.pc += 2;
    }

    fn ld_vx_vy_8xy0(&mut self, op: Opcode) {
        self.reg_v[op.x] = self.reg_v[op.y];
        self.pc += 2;
    }

    fn or_vx_vy_8xy1(&mut self, op: Opcode) {
        self.reg_v[op.x] |= self.reg_v[op.y];
        self.pc += 2;
    }

    fn and_vx_vy_8xy2(&mut self, op: Opcode) {
        self.reg_v[op.x] &= self.reg_v[op.y];
        self.pc += 2;
    }

    fn xor_vx_vy_8xy3(&mut self, op: Opcode) {
        self.reg_v[op.x] ^= self.reg_v[op.y];
        self.pc += 2;
    }

    fn add_vx_vy_8xy4(&mut self, op: Opcode) {
        let tmp = (self.reg_v[op.x] as u16).wrapping_add(self.reg_v[op.y] as u16);
        self.reg_v[0xF] = if tmp > 255 { 1 } else { 0 };
        self.reg_v[op.x] = tmp as u8;
        self.pc += 2;
    }

    fn sub_vx_vy_8xy5(&mut self, op: Opcode) {
        self.reg_v[0xF] = if self.reg_v[op.x] > self.reg_v[op.y] { 1 } else { 0 };
        let tmp = (self.reg_v[op.x] as u16).wrapping_sub(self.reg_v[op.y] as u16);
        self.reg_v[op.x] = tmp as u8;
        self.pc += 2;
    }

    fn shr_vx_8xy6(&mut self, op: Opcode) {
        self.reg_v[0xF] = self.reg_v[op.x] & 0x1;
        self.reg_v[op.x] >>= 1;
        self.pc += 2;
    }

    fn subn_vx_vy_8xy7(&mut self, op: Opcode) {
        self.reg_v[0xF] = if self.reg_v[op.y] > self.reg_v[op.x] { 1 } else { 0 };
        let tmp = (self.reg_v[op.y] as u16).wrapping_sub(self.reg_v[op.x] as u16);
        self.reg_v[op.x] = tmp as u8;
        self.pc += 2;
    }

    fn shl_vx_8xye(&mut self, op: Opcode) {
        self.reg_v[0xF] = (self.reg_v[op.x] & 0x80) >> 7;
        self.reg_v[op.x] <<= 1;
        self.pc += 2;
    }

    fn sne_vx_vy_9xy0(&mut self, op: Opcode) {
        self.pc += if self.reg_v[op.x] != self.reg_v[op.y] { 4 } else { 2 };
    }

    fn ld_i_addr_annn(&mut self, op: Opcode) {
        self.reg_i = op.nnn;
        self.pc += 2;
    }

    fn jp_v0_addr_bnnn(&mut self, op: Opcode) {
        self.pc = op.nnn + (self.reg_v[0] as usize);
    }

    fn rnd_vx_kk_cxkk(&mut self, op: Opcode) {
        let mut rng = rand::thread_rng();
        let tmp = (rng.next_u32() as u8) & op.kk;
        self.reg_v[op.x] = tmp as u8;
        self.pc += 2;
    }

    fn drw_vx_vy_n_dxyn(&mut self, op: Opcode) {
        self.reg_v[0xF] = 0;
        for byte in 0..op.n {
            let y = (self.reg_v[op.y] as usize + byte) % HEIGHT as usize;
            for bit in 0..8 {
                let x = (self.reg_v[op.x] as usize + bit) % WIDTH as usize;
                let color = (self.ram[self.reg_i + byte] >> (7 - bit)) & 1;
                self.reg_v[0xF] |= color & self.vram[y][x];
                self.vram[y][x] ^= color;
            }
        }
        self.vram_update = true;
        self.pc += 2;
    }

    fn skp_vx_ex9e(&mut self, op: Opcode, input: [bool; 16]) {
        let mut skip: bool = false;

        if input[self.reg_v[op.x] as usize] { skip = true; }

        self.pc += if skip { 4 } else { 2 };

    }

    fn sknp_vx_exa1(&mut self, op: Opcode, input: [bool; 16]) {
        let mut skip: bool = false;

        if !input[self.reg_v[op.x] as usize] { skip = true; }

        self.pc += if skip { 4 } else { 2 };

    }

    fn ld_vx_dt_fx07(&mut self, op: Opcode) {
        self.reg_v[op.x] = self.reg_d;
        self.pc += 2;
    }

    fn ld_vx_k_fx0a(&mut self, op: Opcode, input: [bool; 16]) {
        for i in 0..input.len() {
            if input[i] {
                self.reg_v[op.x] = i as u8;
                self.pc += 2;
            }
        }
    }

    fn ld_dt_vx_fx15(&mut self, op: Opcode) {
        self.reg_d = self.reg_v[op.x];
        self.pc += 2;
    }

    fn ld_st_vx_fx18(&mut self, op: Opcode) {
        self.reg_s = self.reg_v[op.x];
        self.pc += 2;
    }

    fn add_i_vx_fx1e(&mut self, op: Opcode) {
        self.reg_i += self.reg_v[op.x] as usize;
        self.reg_v[0xF] = if self.reg_i > 0x0F00 { 1 } else { 0 };
        self.pc += 2;
    }

    fn ld_f_vx_fx29(&mut self, op: Opcode) {
        self.reg_i = 5 * (self.reg_v[op.x] as usize);
        self.pc += 2;
    }

    fn ld_b_vx_fx33(&mut self, op: Opcode) {
        self.ram[self.reg_i] = self.reg_v[op.x] / 100;
        self.ram[self.reg_i + 1] = (self.reg_v[op.x] % 100) / 10;
        self.ram[self.reg_i + 2] = self.reg_v[op.x] % 10;
        self.pc += 2;
    }

    fn ld_i_vx_fx55(&mut self, op: Opcode) {
        for i in 0..op.x + 1 {
            self.ram[self.reg_i + i] = self.reg_v[i];
        }
        self.pc += 2;
    }

    fn ld_vx_i_fx65(&mut self, op: Opcode) {
        for i in 0..op.x + 1 {
             self.reg_v[i] = self.ram[self.reg_i + i];
        }
        self.pc += 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_opcode() {
        let opcode = Opcode::new(0xF333);
        assert_eq!(opcode.opcode, 0xF333);
    }

    #[test]
    fn opcode_nibbles() {
        let opcode = Opcode::new(0xF333);
        assert_eq!(opcode.nibbles, (0xF, 0x3, 0x3, 0x3));
    }

    #[test]
    fn opcode_nnn() {
        let opcode = Opcode::new(0xF333);
        assert_eq!(opcode.nnn, 0x333);
    }

    #[test]
    fn opcode_n() {
        let opcode = Opcode::new(0xF123);
        assert_eq!(opcode.n, 0x3);
    }

    #[test]
    fn opcode_x() {
        let opcode = Opcode::new(0xF123);
        assert_eq!(opcode.x, 0x1);
    }

    #[test]
    fn opcode_y() {
        let opcode = Opcode::new(0xF123);
        assert_eq!(opcode.y, 0x2);
    }

    #[test]
    fn opcode_kk() {
        let opcode = Opcode::new(0xF123);
        assert_eq!(opcode.kk, 0x23);
    }

    #[test]
    fn cpu_v() {
        let cpu = Chip8Cpu::new();
        for i in 0..16 {
            assert_eq!(cpu.reg_v[i], 0);
        }
    }

    #[test]
    fn cpu_i() {
        let cpu = Chip8Cpu::new();
        assert_eq!(cpu.reg_i, 0);
    }

    #[test]
    fn cpu_d() {
        let cpu = Chip8Cpu::new();
        assert_eq!(cpu.reg_d, 0);
    }

    #[test]
    fn cpu_s() {
        let cpu = Chip8Cpu::new();
        assert_eq!(cpu.reg_s, 0);
    }

    #[test]
    fn cpu_pc() {
        let cpu = Chip8Cpu::new();
        assert_eq!(cpu.pc, PROG_START);
    }

    #[test]
    fn cpu_sp() {
        let cpu = Chip8Cpu::new();
        assert_eq!(cpu.sp, 0);
    }

    #[test]
    fn cpu_stack() {
        let cpu = Chip8Cpu::new();
        for i in 0..16 {
            assert_eq!(cpu.stack[i], 0);
        }
    }

    #[test]
    fn cpu_ram() {
        let cpu = Chip8Cpu::new();
        for i in 0..RAM {
            assert_eq!(cpu.ram[i], 0);
        }
    }

    #[test]
    fn cpu_load_rom() {
        let rom_path = "/Users/bryce/dev/chip8/roms/c8games/MAZE";
        let mem = [
            0xA2, 0x1E, 0xC2, 0x01,
            0x32, 0x01, 0xA2, 0x1A,
            0xD0, 0x14, 0x70, 0x04,
            0x30, 0x40, 0x12, 0x00,
            0x60, 0x00, 0x71, 0x04,
            0x31, 0x20, 0x12, 0x00,
            0x12, 0x18, 0x80, 0x40,
            0x20, 0x10, 0x20, 0x40,
            0x80, 0x10
        ];
        let mut cpu = Chip8Cpu::new();
        cpu.load_rom(rom_path);
        for i in 0..34 {
            assert_eq!(mem[i], cpu.ram[PROG_START + i]);
        }
    }

    #[test]
    fn cpu_fetch_opcode() {
        let rom_path = "/Users/bryce/dev/chip8/roms/c8games/MAZE";
        let op = Opcode::new(0xA21E);
        let mut cpu = Chip8Cpu::new();
        cpu.load_rom(rom_path);

        assert_eq!(op, cpu.fetch_opcode());
    }

    #[test]
    fn cpu_test_delay_timer() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_d = 42;
        cpu.tick([false; 16]);
        assert_eq!(cpu.reg_d, 41);
    }

    #[test]
    fn cpu_test_delay_timer_zero() {
        let mut cpu = Chip8Cpu::new();
        assert_eq!(cpu.reg_d, 0);
        cpu.tick([false; 16]);
        assert_eq!(cpu.reg_d, 0);
    }

    #[test]
    fn cpu_test_sound_timer() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_s = 42;
        cpu.tick([false; 16]);
        assert_eq!(cpu.reg_s, 41);
    }

    #[test]
    fn cpu_test_sound_timer_zero() {
        let mut cpu = Chip8Cpu::new();
        assert_eq!(cpu.reg_s, 0);
        cpu.tick([false; 16]);
        assert_eq!(cpu.reg_s, 0);
    }

    #[test]
    #[ignore]
    fn cpu_cls_00e0() {
        panic!("TODO: CLS test");
    }

    #[test]
    fn cpu_ret_00ee() {
        let mut cpu = Chip8Cpu::new();
        cpu.sp = 1;
        cpu.stack[cpu.sp] = 0x500;
        cpu.ret_00ee();
        assert_eq!(cpu.pc, 0x500);
    }

    #[test]
    fn cpu_jp_addr_1nnn() {
        let mut cpu = Chip8Cpu::new();
        cpu.jp_addr_1nnn(Opcode::new(0x1500));
        assert_eq!(cpu.pc, 0x500);
    }

    #[test]
    fn cpu_call_addr_2nnn() {
        let mut cpu = Chip8Cpu::new();
        cpu.call_addr_2nnn(Opcode::new(0x2500));
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[cpu.sp], 0x200);
        assert_eq!(cpu.pc, 0x500);
    }

    #[test]
    fn cpu_se_vx_kk_3xkk_eq() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x42;
        cpu.se_vx_kk_3xkk(Opcode::new(0x3042));
        assert_eq!(cpu.pc, PROG_START + 4);
    }

    #[test]
    fn cpu_se_vx_kk_3xkk_neq() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x43;
        cpu.se_vx_kk_3xkk(Opcode::new(0x3042));
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_sne_vx_kk_4xkk_eq() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x42;
        cpu.sne_vx_kk_4xkk(Opcode::new(0x4042));
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_sne_vx_kk_4xkk_neq() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x43;
        cpu.sne_vx_kk_4xkk(Opcode::new(0x3042));
        assert_eq!(cpu.pc, PROG_START + 4);
    }

    #[test]
    fn cpu_se_vx_vy_5xy0_eq() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x42;
        cpu.reg_v[1] = 0x42;
        cpu.se_vx_vy_5xy0(Opcode::new(0x5010));
        assert_eq!(cpu.pc, PROG_START + 4);
    }

    #[test]
    fn cpu_se_vx_vy_5xy0_neq() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x42;
        cpu.reg_v[1] = 0x43;
        cpu.se_vx_vy_5xy0(Opcode::new(0x5010));
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_ld_vx_kk_6xkk() {
        let mut cpu = Chip8Cpu::new();
        assert_eq!(cpu.reg_v[0], 0x0);
        cpu.ld_vx_kk_6xkk(Opcode::new(0x6042));
        assert_eq!(cpu.reg_v[0], 0x42);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_add_vx_kk_7xkk() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x2;
        cpu.add_vx_kk_7xkk(Opcode::new(0x7040));
        assert_eq!(cpu.reg_v[0], 0x42);
        assert_eq!(cpu.pc, PROG_START + 2);
}

    #[test]
    fn cpu_ld_vx_vy_8xy0() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[1] = 0x42;
        assert_eq!(cpu.reg_v[0], 0x0);
        cpu.ld_vx_vy_8xy0(Opcode::new(0x8010));
        assert_eq!(cpu.reg_v[0], 0x42);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_or_vx_vy_8xy1() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[1] = 0xAB;
        assert_eq!(cpu.reg_v[0], 0x0);
        cpu.or_vx_vy_8xy1(Opcode::new(0x8011));
        assert_eq!(cpu.reg_v[0], 0xAB);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_and_vx_vy_8xy2() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[1] = 0xAB;
        assert_eq!(cpu.reg_v[0], 0x0);
        cpu.and_vx_vy_8xy2(Opcode::new(0x8012));
        assert_eq!(cpu.reg_v[0], 0x0);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_xor_vx_vy_8xy3() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x01;
        cpu.reg_v[1] = 0xAB;
        cpu.xor_vx_vy_8xy3(Opcode::new(0x8013));
        assert_eq!(cpu.reg_v[0], 0xAA);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_add_vx_vy_8xy4_no_carry() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[1] = 0x42;
        cpu.add_vx_vy_8xy4(Opcode::new(0x8014));
        assert_eq!(cpu.reg_v[0], 0x42);
        assert_eq!(cpu.reg_v[0xF], 0x0);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_add_vx_vy_8xy4_carry() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x01;
        cpu.reg_v[1] = 0xFF;
        cpu.add_vx_vy_8xy4(Opcode::new(0x8014));
        assert_eq!(cpu.reg_v[0], 0x0);
        assert_eq!(cpu.reg_v[0xF], 0x1);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_sub_vx_vy_8xy5_no_borrow() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x43;
        cpu.reg_v[1] = 0x01;
        cpu.sub_vx_vy_8xy5(Opcode::new(0x8015));
        assert_eq!(cpu.reg_v[0], 0x42);
        assert_eq!(cpu.reg_v[0xF], 0x1);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_sub_vx_vy_8xy5_borrow() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x01;
        cpu.reg_v[1] = 0x43;
        cpu.sub_vx_vy_8xy5(Opcode::new(0x8015));
        assert_eq!(cpu.reg_v[0], 0xBE);
        assert_eq!(cpu.reg_v[0xF], 0x0);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_shr_vx_8xy6_lsb_one() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x3;
        cpu.shr_vx_8xy6(Opcode::new(0x8016));
        assert_eq!(cpu.reg_v[0xF], 0x1);
        assert_eq!(cpu.reg_v[0], 0x1);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_shr_vx_8xy6_lsb_zero() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x2;
        cpu.shr_vx_8xy6(Opcode::new(0x8016));
        assert_eq!(cpu.reg_v[0xF], 0x0);
        assert_eq!(cpu.reg_v[0], 0x1);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_subn_vx_vy_8xy7_no_borrow() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x01;
        cpu.reg_v[1] = 0x43;
        cpu.subn_vx_vy_8xy7(Opcode::new(0x8017));
        assert_eq!(cpu.reg_v[0], 0x42);
        assert_eq!(cpu.reg_v[0xF], 0x1);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_subn_vx_vy_8xy7_borrow() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x43;
        cpu.reg_v[1] = 0x01;
        cpu.subn_vx_vy_8xy7(Opcode::new(0x8017));
        assert_eq!(cpu.reg_v[0], 0xBE);
        assert_eq!(cpu.reg_v[0xF], 0x0);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_shl_vx_8xye_msb_one() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0xC0;
        cpu.shl_vx_8xye(Opcode::new(0x801E));
        assert_eq!(cpu.reg_v[0xF], 0x1);
        assert_eq!(cpu.reg_v[0], 0x80);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_shl_vx_8xye_msb_zero() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x40;
        cpu.shl_vx_8xye(Opcode::new(0x801E));
        assert_eq!(cpu.reg_v[0xF], 0x0);
        assert_eq!(cpu.reg_v[0], 0x80);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_sne_vx_vy_9xy0_neq() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[1] = 0x1;
        cpu.sne_vx_vy_9xy0(Opcode::new(0x9010));
        assert_eq!(cpu.pc, PROG_START + 4);
    }

    #[test]
    fn cpu_sne_vx_vy_9xy0_eq() {
        let mut cpu = Chip8Cpu::new();
        cpu.sne_vx_vy_9xy0(Opcode::new(0x9010));
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_ld_i_addr_annn() {
        let mut cpu = Chip8Cpu::new();
        cpu.ld_i_addr_annn(Opcode::new(0xA555));
        assert_eq!(cpu.reg_i, 0x555);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_jp_v0_addr_bnnn() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x55;
        cpu.jp_v0_addr_bnnn(Opcode::new(0xB500));
        assert_eq!(cpu.pc, 0x555);
    }

    #[test]
    #[ignore]
    fn cpu_rnd_vx_kk_cxkk() {
        let mut cpu = Chip8Cpu::new();
        cpu.rnd_vx_kk_cxkk(Opcode::new(0xC0FF));
        assert_eq!(cpu.reg_v[0], 0x48);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    #[ignore]
    fn cpu_drw_vx_vy_n_dxyn() {

    }

    #[test]
    #[ignore]
    fn cpu_skp_vx_ex9e() {

    }

    #[test]
    #[ignore]
    fn cpu_sknp_vx_exa1() {

    }

    #[test]
    fn cpu_ld_vx_dt_fx07() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_d = 0x42;
        cpu.ld_vx_dt_fx07(Opcode::new(0xF007));
        assert_eq!(cpu.reg_v[0], 0x42);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    #[ignore]
    fn cpu_ld_vx_key_fx0a() {

    }

    #[test]
    fn cpu_ld_dt_vx_fx15() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x42;
        cpu.ld_dt_vx_fx15(Opcode::new(0xF015));
        assert_eq!(cpu.reg_d, 0x42);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_ld_st_vx_fx18() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x42;
        cpu.ld_st_vx_fx18(Opcode::new(0xF018));
        assert_eq!(cpu.reg_s, 0x42);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_add_i_vx_fx1e() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_i = 0x01;
        cpu.reg_v[0] = 0x41;
        cpu.add_i_vx_fx1e(Opcode::new(0xF01E));
        assert_eq!(cpu.reg_i, 0x42);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_ld_f_vx_fx29() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x2;
        cpu.ld_f_vx_fx29(Opcode::new(0xF029));
        assert_eq!(cpu.reg_i, 0xA);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_ld_b_vx_fx33() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0xEA;
        cpu.ld_b_vx_fx33(Opcode::new(0xF033));
        assert_eq!(cpu.ram[cpu.reg_i], 0x2);
        assert_eq!(cpu.ram[cpu.reg_i + 1], 0x3);
        assert_eq!(cpu.ram[cpu.reg_i + 2], 0x4);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_ld_i_vx_fx55() {
        let mut cpu = Chip8Cpu::new();
        for i in 0..3 {
            cpu.reg_v[i] = 0x1;
        }
        cpu.ld_i_vx_fx55(Opcode::new(0xF355));
        for i in 0..3 {
            assert_eq!(cpu.ram[cpu.reg_i + i], 0x1);
        }
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_ld_vx_i_fx65() {
        let mut cpu = Chip8Cpu::new();
        for i in 0..3 {
            cpu.ram[cpu.reg_i + i] = 0x1;
        }
        cpu.ld_vx_i_fx65(Opcode::new(0xF365));
        for i in 0..3 {
            assert_eq!(cpu.reg_v[i], 0x1);
        }
        assert_eq!(cpu.pc, PROG_START + 2);
    }
}
