use std::fs::File;
use std::io::{Read, Result};
use std::fmt::{Display, Formatter, Result as fmtResult};

extern crate rand;
use rand::{SeedableRng, RngCore, Rng};
extern crate rand_pcg;

const RAM: usize = 4096;
const HEIGHT: usize = 32;
const WIDTH: usize = 64;
const PROG_START: usize = 0x200;

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
    vram: [[u8; WIDTH]; HEIGHT],
    vram_update: bool,
}

impl Chip8Cpu {
    pub fn new() -> Self {
        Chip8Cpu {
            reg_v: [0; 16],
            reg_i: 0,
            reg_d: 0,
            reg_s: 0,
            pc: PROG_START,
            sp: 0,
            stack: [0; 16],
            ram: [0; RAM],
            vram: [[0; WIDTH]; HEIGHT],
            vram_update: false,
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

    pub fn tick(&mut self) {
        self.decode_opcode(self.fetch_opcode());

        if self.reg_d > 0 { self.reg_d -= 1; }
        if self.reg_s > 0 { self.reg_s -= 1; }
    }

    fn decode_opcode(&mut self, op: Opcode) {
        println!("Opcode: {}", op);
        match op.nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.cls_00E0(),
            (0x0, 0x0, 0xE, 0xE) => self.ret_00EE(),
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
            (0x8, _, _, 0xE) => self.shl_vx_8xyE(op),
            (0x9, _, _, 0x0) => self.sne_vx_vy_9xy0(op),
            (0xA, _, _, _) => self.ld_i_addr_Annn(op),
            (0xB, _, _, _) => self.jp_v0_addr_Bnnn(op),
            (0xC, _, _, _) => self.rnd_vx_kk_Cxkk(op),
            (0xD, _, _, _) => self.drw_vx_vy_n_Dxyn(op),
            (0xF, _, 0x0, 0x7) => self.ld_vx_dt_Fx07(op),
            (0xF, _, 0x1, 0x5) => self.ld_dt_vx_Fx15(op),
            (0xF, _, 0x1, 0x8) => self.ld_st_vx_Fx18(op),
            (0xF, _, 0x1, 0xE) => self.add_i_vx_Fx1E(op),
            (0xF, _, 0x2, 0x9) => self.ld_f_vx_Fx29(op),
            (0xF, _, 0x3, 0x3) => self.ld_b_vx_Fx33(op),
            (0xF, _, 0x5, 0x5) => self.ld_i_vx_Fx55(op),
            (0xF, _, 0x6, 0x5) => self.ld_vx_i_Fx65(op),
            _ => {
                println!("Unimplemented opcode: {}", op);
                self.pc += 2;
            }
        }
    }

    fn cls_00E0(&self) {
        // TODO
    }

    fn ret_00EE(&mut self) {
        self.pc = self.stack[self.sp];
        self.sp -= 1;
    }

    fn jp_addr_1nnn(&mut self, op: Opcode) {
        self.pc = op.nnn;
    }

    fn call_addr_2nnn(&mut self, op: Opcode) {
        self.sp += 1;
        self.stack[self.sp] = self.pc;
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

    fn shl_vx_8xyE(&mut self, op: Opcode) {
        self.reg_v[0xF] = (self.reg_v[op.x] & 0x80) >> 7;
        self.reg_v[op.x] <<= 1;
        self.pc += 2;
    }

    fn sne_vx_vy_9xy0(&mut self, op: Opcode) {
        self.pc += if self.reg_v[op.x] != self.reg_v[op.y] { 4 } else { 2 };
    }

    fn ld_i_addr_Annn(&mut self, op: Opcode) {
        self.reg_i = op.nnn;
        self.pc += 2;
    }

    fn jp_v0_addr_Bnnn(&mut self, op: Opcode) {
        self.pc = op.nnn + (self.reg_v[0] as usize);
    }

    fn rnd_vx_kk_Cxkk(&mut self, op: Opcode) {
        let mut rng = rand::thread_rng();
        let tmp = (rng.next_u32() as u8) & op.kk;
        self.reg_v[op.x] = tmp as u8;
        self.pc += 2;
    }

    fn drw_vx_vy_n_Dxyn(&mut self, op: Opcode) -> () {
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

    fn ld_vx_dt_Fx07(&mut self, op: Opcode) {
        self.reg_v[op.x] = self.reg_d;
        self.pc += 2;
    }

    fn ld_dt_vx_Fx15(&mut self, op: Opcode) {
        self.reg_d = self.reg_v[op.x];
        self.pc += 2;
    }

    fn ld_st_vx_Fx18(&mut self, op: Opcode) {
        self.reg_s = self.reg_v[op.x];
        self.pc += 2;
    }

    fn add_i_vx_Fx1E(&mut self, op: Opcode) {
        let tmp = (self.reg_i as u16).wrapping_add(self.reg_v[op.x] as u16);
        self.reg_i = tmp as usize;
        self.pc += 2;
    }

    fn ld_f_vx_Fx29(&mut self, op: Opcode) {
        self.reg_i = 5 * (self.reg_v[op.x] as usize);
        self.pc += 2;
    }

    fn ld_b_vx_Fx33(&mut self, op: Opcode) {
        self.ram[self.reg_i] = self.reg_v[op.x] / 100;
        self.ram[self.reg_i + 1] = (self.reg_v[op.x] / 10) % 10;
        self.ram[self.reg_i + 2] = self.reg_v[op.x] % 10;
        self.pc += 2;
    }

    fn ld_i_vx_Fx55(&mut self, op: Opcode) {
        for i in 0..op.x {
            self.ram[self.reg_i + i] = self.reg_v[i];
        }
        self.pc += 2;
    }

    fn ld_vx_i_Fx65(&mut self, op: Opcode) {
        for i in 0..op.x {
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
        cpu.tick();
        assert_eq!(cpu.reg_d, 41);
    }

    #[test]
    fn cpu_test_delay_timer_zero() {
        let mut cpu = Chip8Cpu::new();
        assert_eq!(cpu.reg_d, 0);
        cpu.tick();
        assert_eq!(cpu.reg_d, 0);
    }

    #[test]
    fn cpu_test_sound_timer() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_s = 42;
        cpu.tick();
        assert_eq!(cpu.reg_s, 41);
    }

    #[test]
    fn cpu_test_sound_timer_zero() {
        let mut cpu = Chip8Cpu::new();
        assert_eq!(cpu.reg_s, 0);
        cpu.tick();
        assert_eq!(cpu.reg_s, 0);
    }

    #[test]
    #[ignore]
    fn cpu_cls_00E0() {
        panic!("TODO: CLS test");
    }

    #[test]
    fn cpu_ret_00EE() {
        let mut cpu = Chip8Cpu::new();
        cpu.sp = 1;
        cpu.stack[cpu.sp] = 0x500;
        cpu.ret_00EE();
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
    fn cpu_shl_vx_8xyE_msb_one() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0xC0;
        cpu.shl_vx_8xyE(Opcode::new(0x801E));
        assert_eq!(cpu.reg_v[0xF], 0x1);
        assert_eq!(cpu.reg_v[0], 0x80);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_shl_vx_8xyE_msb_zero() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x40;
        cpu.shl_vx_8xyE(Opcode::new(0x801E));
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
    fn cpu_ld_i_addr_Annn() {
        let mut cpu = Chip8Cpu::new();
        cpu.ld_i_addr_Annn(Opcode::new(0xA555));
        assert_eq!(cpu.reg_i, 0x555);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_jp_v0_addr_Bnnn() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x55;
        cpu.jp_v0_addr_Bnnn(Opcode::new(0xB500));
        assert_eq!(cpu.pc, 0x555);
    }

    #[test]
    #[ignore]
    fn cpu_rnd_vx_kk_Cxkk() {
        let mut cpu = Chip8Cpu::new();
        cpu.rnd_vx_kk_Cxkk(Opcode::new(0xC0FF));
        assert_eq!(cpu.reg_v[0], 0x48);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    #[ignore]
    fn cpu_drw_vx_vy_n_Dxyn() {

    }

    #[test]
    #[ignore]
    fn cpu_skp_vx_Ex9E() {

    }

    #[test]
    #[ignore]
    fn cpu_sknp_vx_ExA1() {

    }

    #[test]
    fn cpu_ld_vx_dt_Fx07() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_d = 0x42;
        cpu.ld_vx_dt_Fx07(Opcode::new(0xF007));
        assert_eq!(cpu.reg_v[0], 0x42);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    #[ignore]
    fn cpu_ld_vx_key_Fx0A() {

    }

    #[test]
    fn cpu_ld_dt_vx_Fx15() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x42;
        cpu.ld_dt_vx_Fx15(Opcode::new(0xF015));
        assert_eq!(cpu.reg_d, 0x42);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_ld_st_vx_Fx18() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x42;
        cpu.ld_st_vx_Fx18(Opcode::new(0xF018));
        assert_eq!(cpu.reg_s, 0x42);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_add_i_vx_Fx1E() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_i = 0x01;
        cpu.reg_v[0] = 0x41;
        cpu.add_i_vx_Fx1E(Opcode::new(0xF01E));
        assert_eq!(cpu.reg_i, 0x42);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_ld_f_vx_Fx29() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0x2;
        cpu.ld_f_vx_Fx29(Opcode::new(0xF029));
        assert_eq!(cpu.reg_i, 0xA);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_ld_b_vx_Fx33() {
        let mut cpu = Chip8Cpu::new();
        cpu.reg_v[0] = 0xEA;
        cpu.ld_b_vx_Fx33(Opcode::new(0xF033));
        assert_eq!(cpu.ram[cpu.reg_i], 0x2);
        assert_eq!(cpu.ram[cpu.reg_i + 1], 0x3);
        assert_eq!(cpu.ram[cpu.reg_i + 2], 0x4);
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_ld_i_vx_Fx55() {
        let mut cpu = Chip8Cpu::new();
        for i in 0..3 {
            cpu.reg_v[i] = 0x1;
        }
        cpu.ld_i_vx_Fx55(Opcode::new(0xF355));
        for i in 0..3 {
            assert_eq!(cpu.ram[cpu.reg_i + i], 0x1);
        }
        assert_eq!(cpu.pc, PROG_START + 2);
    }

    #[test]
    fn cpu_ld_vx_i_Fx65() {
        let mut cpu = Chip8Cpu::new();
        for i in 0..3 {
            cpu.ram[cpu.reg_i + i] = 0x1;
        }
        cpu.ld_vx_i_Fx65(Opcode::new(0xF365));
        for i in 0..3 {
            assert_eq!(cpu.reg_v[i], 0x1);
        }
        assert_eq!(cpu.pc, PROG_START + 2);
    }
}

// use rand::Rng;
//
// extern crate sdl2;
// use sdl2::event::Event;
// use sdl2::keyboard::{KeyboardState, Keycode, Scancode};
//

//
// mod display;
// use display::Chip8Display;
//
// const FPS: u32 = 20;
//
// impl Cpu<'_> {
//     pub fn execute_rom(&mut self) {
//         let mut fps_clock = FpsClock::new(FPS);
//
//         'game_loop:loop {
//             for event in self.display.event_pump.poll_iter() {
//                 match event {
//                     Event::KeyDown {
//                         keycode: Some(Keycode::Escape), ..
//                     } => { break 'game_loop }
//                     _ => {}
//                 }
//             }
//
//             let opcode = self.opcode();
//
//             self.execute_opcode(opcode);
//
//             if self.reg_d > 0 {
//                 self.reg_d -= 1;
//             }
//             if self.reg_s > 0 {
//                 self.reg_s -= 1;
//             }
//             if self.display.vram_changed {
//                 self.display.draw();
//             }
//
//             fps_clock.tick();
//         }
//     }
//
//     fn execute_opcode(&mut self, opcode: Opcode) {
//         println!("{}", opcode);
//         match opcode.nibbles.0 {
//             0x0 => self.opcode_0x0(opcode),
//             0x1 => self.opcode_0x1(opcode),
//             0x2 => self.opcode_0x2(opcode),
//             0x3 => self.opcode_0x3(opcode),
//             0x4 => self.opcode_0x4(opcode),
//             0x5 => self.opcode_0x5(opcode),
//             0x6 => self.opcode_0x6(opcode),
//             0x7 => self.opcode_0x7(opcode),
//             0x8 => self.opcode_0x8(opcode),
//             0x9 => self.opcode_0x9(opcode),
//             0xA => self.opcode_0xA(opcode),
//             0xB => self.opcode_0xB(opcode),
//             0xC => self.opcode_0xC(opcode),
//             0xD => self.opcode_0xD(opcode),
//             0xE => self.opcode_0xE(opcode),
//             0xF => self.opcode_0xF(opcode),
//             _ => {
//                 println!("{}", Cpu::unimplemented_opcode(opcode));
//                 self.pc += 2;
//             }
//         }
//     }
//
//     fn opcode_0x0(&mut self, opcode: Opcode) {
//         match opcode.nibbles.2 {
//             0xE => match opcode.nibbles.3 {
//                 0x0 => {
//                     self.display.vram = [[0; WIDTH as usize]; HEIGHT as usize];
//                     self.display.vram_changed = true;
//                 },
//                 0xE => {
//                     self.sp -= 1;
//                     self.pc = self.stack[self.sp];
//                 },
//                 _ => {
//                     println!("{}", Cpu::unimplemented_opcode(opcode));
//                 }
//             }
//             _ => {
//                 println!("{}", Cpu::unimplemented_opcode(opcode));
//             }
//         }
//         self.pc += 2;
//     }
//
//     fn opcode_0x1(&mut self, opcode: Opcode) {
//         self.pc = opcode.nnn;
//     }
//
//     fn opcode_0x2(&mut self, opcode: Opcode) {
//         self.stack[self.sp] = self.pc + 2;
//         self.sp += 1;
//         self.pc = opcode.nnn;
//     }
//
//     fn opcode_0x3(&mut self, opcode: Opcode) {
//         if self.reg_v[opcode.x] == opcode.kk {
//             self.pc += 4;
//         } else {
//             self.pc += 2;
//         }
//     }
//
//     fn opcode_0x4(&mut self, opcode: Opcode) {
//         if self.reg_v[opcode.x] != opcode.kk {
//             self.pc += 4;
//         } else {
//             self.pc += 2;
//         }
//     }
//
//     fn opcode_0x5(&mut self, opcode: Opcode) {
//         if self.reg_v[opcode.x] == self.reg_v[opcode.y] {
//             self.pc += 4;
//         } else {
//             self.pc += 2;
//         }
//     }
//
//     fn opcode_0x6(&mut self, opcode: Opcode) {
//         self.reg_v[opcode.x] = opcode.kk;
//         self.pc += 2;
//     }
//
//     fn opcode_0x7(&mut self, opcode: Opcode) {
//         let tmp = self.reg_v[opcode.x] as u16 + opcode.kk as u16;
//         self.reg_v[opcode.x] = tmp as u8;
//         self.pc += 2;
//     }
//
//     fn opcode_0x8(&mut self, opcode: Opcode) {
//         match opcode.nibbles.3 {
//             0x0 => self.reg_v[opcode.x] = self.reg_v[opcode.y],
//             0x1 => self.reg_v[opcode.x] |= self.reg_v[opcode.y],
//             0x2 => self.reg_v[opcode.x] &= self.reg_v[opcode.y],
//             0x3 => self.reg_v[opcode.x] ^= self.reg_v[opcode.y],
//             0x4 => {
//                 let tmp = self.reg_v[opcode.x] as u16 + self.reg_v[opcode.y] as u16;
//                 self.reg_v[opcode.x] = tmp as u8;
//                 self.reg_v[0xF] = if tmp > 0xFF { 1 } else { 0 };
//             },
//             0x5 => {
//                 self.reg_v[0xF] = if self.reg_v[opcode.x] > self.reg_v[opcode.y] { 1 } else { 0 };
//                 self.reg_v[opcode.x] = self.reg_v[opcode.x].wrapping_sub(self.reg_v[opcode.y]);
//             },
//             0x6 => {
//                 self.reg_v[0xF] = self.reg_v[opcode.x] & 0x1;
//                 self.reg_v[opcode.x] >>= 1;
//             },
//             0x7 => {
//                 self.reg_v[0xF] = if self.reg_v[opcode.x] < self.reg_v[opcode.y] { 1 } else { 0 };
//                 self.reg_v[opcode.x] = self.reg_v[opcode.y].wrapping_sub(self.reg_v[opcode.x]);
//             },
//             0xE => {
//                 self.reg_v[0xF] = (self.reg_v[opcode.x] & 0b10000000) >> 7;
//                 self.reg_v[opcode.x] <<= 1;
//             },
//             _ => {
//                 println!("{}", Cpu::unimplemented_opcode(opcode));
//             }
//         }
//         self.pc += 2;
//     }
//
//     fn opcode_0x9(&mut self, opcode: Opcode) {
//         if self.reg_v[opcode.x] != self.reg_v[opcode.y] {
//             self.pc += 4;
//         } else {
//             self.pc += 2;
//         }
//     }
//
//     #[allow(non_snake_case)]
//     fn opcode_0xA(&mut self, opcode: Opcode) {
//         self.reg_i = opcode.nnn;
//         self.pc += 2;
//     }
//
//     #[allow(non_snake_case)]
//     fn opcode_0xB(&mut self, opcode: Opcode) {
//         self.pc = opcode.nnn + self.reg_v[0x0] as usize;
//     }
//
//     #[allow(non_snake_case)]
//     fn opcode_0xC(&mut self, opcode: Opcode) {
//         let mut rnd = rand::thread_rng();
//         self.reg_v[opcode.x] = rnd.gen::<u8>() & opcode.kk;
//         self.pc += 2;
//     }
//
//     #[allow(non_snake_case)]
//     fn opcode_0xD(&mut self, opcode: Opcode) {
//         self.reg_v[0xF] = 0;
//         for byte in 0..opcode.n {
//             let y = (self.reg_v[opcode.y] as usize + byte) % HEIGHT as usize;
//             for bit in 0..8 {
//                 let x = (self.reg_v[opcode.x] as usize + bit) % WIDTH as usize;
//                 let color = (self.ram[self.reg_i + byte] >> (7 - bit)) & 1;
//                 self.reg_v[0xF] |= color & self.display.vram[y][x];
//                 self.display.vram[y][x] ^= color;
//             }
//         }
//         self.display.vram_changed = true;
//         self.pc += 2;
//     }
//
//     #[allow(non_snake_case)]
//     fn opcode_0xE(&mut self, opcode: Opcode) {
//         match opcode.nibbles.2 {
//             0x9 => match opcode.nibbles.3 {
//                 0xE => {
//                     let mut skip = false;
//                     let keyboard = KeyboardState::new(&self.display.event_pump);
//
//                     match self.reg_v[opcode.x] {
//                         0x1 => {
//                             skip = keyboard.is_scancode_pressed(Scancode::Num1);
//                         },
//                         0x2 => {
//                             skip = keyboard.is_scancode_pressed(Scancode::Num2);
//                         },
//                         0x3 => {
//                             skip = keyboard.is_scancode_pressed(Scancode::Num3);
//                         },
//                         0x4 => {
//                             skip = keyboard.is_scancode_pressed(Scancode::Q);
//                         },
//                         0x5 => {
//                             skip = keyboard.is_scancode_pressed(Scancode::W);
//                         },
//                         0x6 => {
//                             skip = keyboard.is_scancode_pressed(Scancode::E);
//                         },
//                         0x7 => {
//                             skip = keyboard.is_scancode_pressed(Scancode::A);
//                         },
//                         0x8 => {
//                             skip = keyboard.is_scancode_pressed(Scancode::S);
//                         },
//                         0x9 => {
//                             skip = keyboard.is_scancode_pressed(Scancode::D);
//                         },
//                         0x0 => {
//                             skip = keyboard.is_scancode_pressed(Scancode::X);
//                         },
//                         0xA => {
//                             skip = keyboard.is_scancode_pressed(Scancode::Z);
//                         },
//                         0xB => {
//                             skip = keyboard.is_scancode_pressed(Scancode::C);
//                         },
//                         0xC => {
//                             skip = keyboard.is_scancode_pressed(Scancode::Num4);
//                         },
//                         0xD => {
//                             skip = keyboard.is_scancode_pressed(Scancode::R);
//                         },
//                         0xE => {
//                             skip = keyboard.is_scancode_pressed(Scancode::F);
//                         },
//                         0xF => {
//                             skip = keyboard.is_scancode_pressed(Scancode::V);
//                         },
//                         _ => println!("{}", Cpu::unimplemented_opcode(opcode))
//                     }
//
//                     self.pc += if skip { 4 } else { 2 };
//                 },
//                 _ => println!("{}", Cpu::unimplemented_opcode(opcode))
//             },
//             0xA => match opcode.nibbles.3 {
//                 0x1 => {
//                     let mut skip = false;
//                     let keyboard = KeyboardState::new(&self.display.event_pump);
//
//                     match self.reg_v[opcode.x] {
//                         0x1 => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::Num1);
//                         },
//                         0x2 => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::Num2);
//                         },
//                         0x3 => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::Num3);
//                         },
//                         0x4 => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::Q);
//                         },
//                         0x5 => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::W);
//                         },
//                         0x6 => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::E);
//                         },
//                         0x7 => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::A);
//                         },
//                         0x8 => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::S);
//                         },
//                         0x9 => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::D);
//                         },
//                         0x0 => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::X);
//                         },
//                         0xA => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::Z);
//                         },
//                         0xB => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::C);
//                         },
//                         0xC => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::Num4);
//                         },
//                         0xD => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::R);
//                         },
//                         0xE => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::F);
//                         },
//                         0xF => {
//                             skip = !keyboard.is_scancode_pressed(Scancode::V);
//                         },
//                         _ => println!("{}", Cpu::unimplemented_opcode(opcode))
//                     }
//
//                     self.pc += if skip { 4 } else { 2 };
//                 },
//                 _ => println!("{}", Cpu::unimplemented_opcode(opcode))
//             }
//             _ => println!("{}", Cpu::unimplemented_opcode(opcode))
//         }
//     }
//
//     #[allow(non_snake_case)]
//     fn opcode_0xF(&mut self, opcode: Opcode) {
//         match opcode.nibbles.2 {
//             0x0 => match opcode.nibbles.3 {
//                 0x7 => self.reg_v[opcode.x] = self.reg_d,
//                 0xA => {
//                     'key_loop:loop {
//                         for event in self.display.event_pump.poll_iter() {
//                             match event {
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::Num1), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0x1;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::Num2), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0x2;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::Num3), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0x3;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::Num4), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0xC;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::Q), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0x4;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::W), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0x5;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::E), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0x6;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::R), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0xD;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::A), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0x7;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::S), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0x8;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::D), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0x9;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::F), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0xE;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::Z), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0xA;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::X), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0x0;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::C), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0xB;
//                                     break 'key_loop
//                                 },
//                                 Event::KeyDown {
//                                     keycode: Some(Keycode::V), ..
//                                 } => {
//                                     self.reg_v[opcode.x] = 0xF;
//                                     break 'key_loop
//                                 },
//                                 _ => {}
//                             }
//                         }
//                     }
//                 }
//                 _ => println!("{}", Cpu::unimplemented_opcode(opcode))
//             },
//             0x1 => match opcode.nibbles.3 {
//                 0x5 => self.reg_d = self.reg_v[opcode.x],
//                 0x8 => self.reg_s = self.reg_v[opcode.x],
//                 0xE => {
//                     self.reg_i += self.reg_v[opcode.x] as usize;
//                     self.reg_v[0xF] = if self.reg_i > 0xF00 { 1 } else { 0 };
//                 },
//                 _ => println!("{}", Cpu::unimplemented_opcode(opcode))
//             },
//             0x2 => match opcode.nibbles.3 {
//                 0x9 => {
//                     let addr = 5 * (self.reg_v[opcode.x] as usize);
//                     self.reg_i = addr;
//                 },
//                 _ => println!("{}", Cpu::unimplemented_opcode(opcode))
//             }
//             0x3 => match opcode.nibbles.3 {
//                 0x3 => {
//                     // let x = self.reg_v[opcode.x];
//                     // let hun = x / 100;
//                     // let ten = (x % 100) / 10;
//                     // let one = x % 10;
//                     let i = self.reg_i;
//                     self.ram[i] = self.reg_v[opcode.x] / 100;
//                     self.ram[i+1] = (self.reg_v[opcode.x] % 100) / 10;
//                     self.ram[i+2] = self.reg_v[opcode.x] % 10;
//                 },
//                 _ => println!("{}", Cpu::unimplemented_opcode(opcode))
//             },
//             0x5 => match opcode.nibbles.3 {
//                 0x5 => {
//                     let x = opcode.x;
//                     let i = self.reg_i;
//                     for idx in 0..x + 1{
//                         self.ram[i + idx] = self.reg_v[idx];
//                     }
//                 },
//                 _ => println!("{}", Cpu::unimplemented_opcode(opcode))
//             },
//             0x6 => match opcode.nibbles.3 {
//                 0x5 => {
//                     let x = opcode.x;
//                     let i = self.reg_i;
//                     for idx in 0..x + 1 {
//                         self.reg_v[idx] = self.ram[i + idx];
//                     }
//                 },
//                 _ => println!("{}", Cpu::unimplemented_opcode(opcode))
//             },
//             _ => println!("{}", Cpu::unimplemented_opcode(opcode))
//         }
//
//         self.pc += 2;
//     }
//
//     fn unimplemented_opcode(opcode: Opcode) -> String {
//         format!("Unimplemented opcode {}", opcode)
//     }
// }
