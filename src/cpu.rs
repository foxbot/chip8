use std::fmt::{Debug, Formatter, Result};

//pub const TIME_PER_UPDATE: f64 = 1000.0 / 60.0;
pub const GFX_COLS: usize = 64;
pub const GFX_ROWS: usize = 32;
//const MAX_ROM_SIZE: usize = 0x1000 - 0x200;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Cpu {
    mem: [u8; 4096],
    pub gfx: [[u8; GFX_COLS]; GFX_ROWS],
    reg: [u8; 16],
    op: u16,
    idx: u16,
    pc: usize,
    del_timer: u8,
    snd_timer: u8,
    stack: [usize; 16],
    sp: usize,
    pub key: [u8; 16],
    pub draw: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut cpu = Cpu {
            mem: [0; 4096],
            gfx: [[0; GFX_COLS]; GFX_ROWS],
            reg: [0; 16],
            op: 0,
            idx: 0,
            pc: 0x200,
            del_timer: 0,
            snd_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],
            draw: false,
        };

        // initialize font
        for i in 0..80 {
            cpu.mem[i] = FONT[i];
        }

        cpu
    }

    pub fn load_rom(&mut self, path: &str) {
        use std::fs::File;
        use std::io::prelude::*;

        let mut f = File::open(path).unwrap();
        let mut rom: [u8; 3584] = [0; 3584];
        f.read(&mut rom).unwrap();

        let size = rom.len();
        for i in 0..size {
            self.mem[0x200 + i] = rom[i];
        }

        println!("loaded rom.");
    }

    pub fn cycle(&mut self) {
        let op: u16 = ((self.mem[self.pc] as u16) << 8) | (self.mem[self.pc + 1] as u16);
        self.op = op; // TOOD: needed?

        // clear draw flag; it should only last for one cycle
        self.draw = false;

        match op & 0xF000 {
            // 0x0NNN SYS addr: system reserved
            0x0000 => {
                match op & 0x0FFF {
                    // CLS clear screen
                    0x00E0 => {
                        self.gfx = [[0; GFX_COLS]; GFX_ROWS];
                        self.draw = true;
                        self.pc = self.pc + 2;
                    }
                    // RET return from subroutine
                    0x00EE => {
                        if self.sp > 0 {
                            self.sp = self.sp - 1;
                        }
                        let dst = self.stack[self.sp];
                        self.pc = dst;
                    }
                    _ => {
                        panic!("unhandled opcode {:4X} at {:4X}", op, self.pc);
                    }
                }
            }
            // 0x1NNN JP addr: jump to
            0x1000 => {
                let dst = op & 0x0FFF;
                self.pc = dst.into();
            }
            // 0x2NNN CALL addr: subroutine
            0x2000 => {
                self.stack[self.sp] = self.pc + 2;
                self.sp = self.sp + 1;

                let dst = op & 0x0FFF;
                self.pc = dst.into();
            }
            // 0x3xkk SE Vx byte: skip if Vx == kk
            0x3000 => {
                let reg = ((op & 0x0F00) >> 8) as usize;
                let val = (op & 0x00FF) as u8;
                if self.reg[reg] == val {
                    self.pc = self.pc + 4;
                }
                self.pc = self.pc + 2;
            }
            // 0x4xkk SNE Vx byte: skip if Vx != kk
            0x4000 => {
                let reg = ((op & 0x0F00) >> 8) as usize;
                let val = (op & 0x00FF) as u8;
                if self.reg[reg] != val {
                    self.pc = self.pc + 4;
                }
                self.pc = self.pc + 2;
            }
            // 0x5xy0 SE Vx, Vy: skip if Vx == Vy
            0x5000 => {
                let vx = ((op & 0x0F00) >> 8) as usize;
                let vy = ((op & 0x00F0) >> 4) as usize;
                if self.reg[vx] == self.reg[vy] {
                    self.pc = self.pc + 4;
                }
                self.pc = self.pc + 2;
            }
            // 0x6xkk LD Vx byte: set Vx = kk
            0x6000 => {
                let reg = ((op & 0x0F00) >> 8) as usize;
                let val = (op & 0x00FF) as u8;
                self.reg[reg] = val;
                self.pc = self.pc + 2;
            }
            // 0x7xkk ADD Vx byte: Vx += kk
            // TODO: see what to do with an overflow here (will panic currently)
            0x7000 => {
                let reg = ((op & 0x0F00) >> 8) as usize;
                let val = (op & 0x00FF) as u8;
                self.reg[reg] = self.reg[reg].saturating_add(val);
                self.pc = self.pc + 2;
            }
            // 0x8xyz Register Arithmetic
            0x8000 => {
                match op & 0x000F {
                    // 0x8xy0 LD Vx, Vy: set Vx = Vy
                    0x0 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        let vy = ((op & 0x00F0) >> 4) as usize;
                        self.reg[vx] = self.reg[vy];
                    }
                    // 0x8xy1 OR Vx, Vy: set Vx = Vx | Vy
                    0x1 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        let vy = ((op & 0x00F0) >> 4) as usize;
                        self.reg[vx] = self.reg[vx] | self.reg[vy];
                    }
                    // 0x8xy2 AND Vx, Vy: set Vx = Vx & Vy
                    0x2 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        let vy = ((op & 0x00F0) >> 4) as usize;
                        self.reg[vx] = self.reg[vx] & self.reg[vy];
                    }
                    // 0x8xy3 XOR Vx, Vy: set Vx = Vx ^ Vy
                    0x3 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        let vy = ((op & 0x00F0) >> 4) as usize;
                        self.reg[vx] = self.reg[vx] ^ self.reg[vy];
                    }
                    // 0x8xy4 ADD Vx, Vy: set Vx = Vy, Vf = carry
                    0x4 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        let vy = ((op & 0x00F0) >> 4) as usize;
                        let x = self.reg[vx];
                        let y = self.reg[vy];

                        if let None = x.checked_add(y) {
                            self.reg[vx] = x.saturating_add(y);
                            self.reg[0xF] = 1;
                        } else {
                            self.reg[vx] = x + y;
                            self.reg[0xF] = 0;
                        }
                    }
                    // 0x8xy5 SUB Vx, Vy: set Vx = Vx - Vy, Vf = (Vx > Vy)
                    0x5 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        let vy = ((op & 0x00F0) >> 4) as usize;
                        let x = self.reg[vx];
                        let y = self.reg[vy];

                        if x > y {
                            self.reg[vx] = x - y;
                            self.reg[0xF] = 1;
                        } else {
                            self.reg[vx] = x.saturating_sub(y);
                            self.reg[0xF] = 0;
                        }
                    }
                    // 0x8xy6 SHR Vx: Vx >>= 1; Vf = Vx & 0x01
                    0x6 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        let x = self.reg[vx];
                        self.reg[0xF] = x & 0x01;
                        self.reg[vx] = x >> 1;
                    }
                    // 0x8xy7 SUBN Vx, Vy; Vx = Vy - Vx, Vf = (Vy > Vx)
                    0x7 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        let vy = ((op & 0x00F0) >> 4) as usize;
                        let x = self.reg[vx];
                        let y = self.reg[vy];

                        if y > x {
                            self.reg[vx] = y - x;
                            self.reg[0xF] = 1;
                        } else {
                            self.reg[vx] = y.saturating_sub(x);
                            self.reg[0xF] = 0;
                        }
                    }
                    // 0x8xyE SHL Vx; Vx <<= 1; Vf = Vx & 0x80
                    0xE => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        let x = self.reg[vx];
                        self.reg[0xF] = x & 0x80;
                        self.reg[vx] = x >> 1;
                    }
                    _ => {}
                }
                self.pc = self.pc + 2;
            }
            // 0x5xy0 SNE Vx, Vy: skip if Vx != Vy
            0x9000 => {
                let vx = ((op & 0x0F00) >> 8) as usize;
                let vy = ((op & 0x00F0) >> 4) as usize;
                if self.reg[vx] != self.reg[vy] {
                    self.pc = self.pc + 4;
                }
                self.pc = self.pc + 2;
            }
            // 0xANNN LDI addr; set I register to NNN
            0xA000 => {
                let val = op & 0x0FFF;
                self.idx = val;
                self.pc = self.pc + 2;
            }
            // 0xBNNN JP V0, addr; pc = v0 + nnn
            0xB000 => {
                let val = op & 0x0FFF;
                self.pc = (self.reg[0] as u16 + val) as usize;
            }
            // 0xCxkk RND Vx, byte: Vx = (rng & kk)
            0xC000 => {
                // TODO: random number generator
                let rand = 4;
                let val = (op & 0x00FF) as u8;
                let vx = ((op & 0x0F00) >> 8) as usize;
                self.reg[vx] = val + rand;
                self.pc = self.pc + 2;
            }
            // 0xDxyn DRW Vx, Vy, nibble: display
            0xD000 => {
                let x = ((op & 0x0F00) >> 8) as usize;
                let y = ((op & 0x00F0) >> 4) as usize;
                let height = (op & 0x000F) as usize;
                let i = self.idx as usize;

                self.reg[0xF] = 0;
                for line in 0..height {
                    let pixel = self.mem[i + line];
                    for col in 0..8 {
                        if (pixel & (0x80 >> col)) != 0 {
                            let mut tx = x + col;
                            if tx > GFX_COLS {
                                tx = tx - GFX_COLS;
                            }
                            let ty = y + line;
                            if self.gfx[ty][tx] == 1 {
                                self.reg[0xF] = 1;
                            }
                            self.gfx[ty][tx] ^= 1;
                        }
                    }
                }

                self.draw = true;
                self.pc = self.pc + 2;
            }
            // 0xExzz Input Handling
            0xE000 => {
                match op & 0x00FF {
                    // 0xEx9E SKP Kx: skip if Kx depressed
                    0x009E => {
                        let kx = ((op & 0x0F00) >> 8) as usize;
                        if self.key[kx] == 1 {
                            self.pc = self.pc + 4;
                        }
                        self.pc = self.pc + 2;
                    }
                    // 0xEx9E SKNP Kx: skip if Kx not pressed
                    0x00A1 => {
                        let kx = ((op & 0x0F00) >> 8) as usize;
                        if self.key[kx] == 0 {
                            self.pc = self.pc + 4;
                        }
                        self.pc = self.pc + 2;
                    }
                    _ => {
                        panic!("unhandled opcode {:4X} at {:4X}", op, self.pc);
                    }
                }
            }
            // Fxzz: multi purpose
            0xF000 => {
                match op & 0x00FF {
                    // 0xFx07 LD Vx, DT; Vx = DT
                    0x0007 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        self.reg[vx] = self.del_timer;
                        self.pc = self.pc + 2;
                    }
                    // 0xFx0A LD Vx, K; wait for keypress, store key idx to Vx
                    0x000A => {
                        for i in 0..15 {
                            if self.key[i] == 1 {
                                let vx = ((op & 0x0F00) >> 8) as usize;
                                self.reg[vx] = i as u8;
                                self.pc = self.pc + 2;
                            }
                        }
                        // don't increment pc, stay on this op until a key is pressed
                    }
                    // 0xFx15 LD DT, Vx; DT = Vx
                    0x0015 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        self.del_timer = self.reg[vx];
                        self.pc = self.pc + 2;
                    }
                    // 0xFx18 LD ST, Vx; ST = Vx
                    0x0018 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        self.snd_timer = self.reg[vx];
                        self.pc = self.pc + 2;
                    }
                    // 0xFx1E ADD I, Vx; I += Vx
                    0x001E => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        self.idx = self.idx + (self.reg[vx] as u16);
                        self.pc = self.pc + 2;
                    }
                    // 0xFx29 LD F, Vx; I = (loc of char in fontset)
                    0x0029 => {
                        let c = (op & 0x0F00) >> 8;
                        self.idx = c * 5;
                        self.pc = self.pc + 2;
                    }
                    // 0xFx33 LD B, Vx; I[0, 1, 2] = Vx (100, 10, 1)
                    0x0033 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        let i = self.idx as usize;
                        let val = self.reg[vx];

                        self.mem[i] = val / 100;
                        self.mem[i + 1] = (val % 100) / 10;
                        self.mem[i + 2] = (val % 100) % 10;

                        self.pc = self.pc + 2;
                    }
                    // 0xFx55 LD [I], Vx; store registers to Vx into mem at I
                    0x0055 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        let i = self.idx as usize;

                        for o in 0..vx {
                            self.mem[i + o] = self.reg[o];
                        }
                        self.pc = self.pc + 2;
                    }
                    // 0xFx65 LD Vx, [I]; write registers to Vx from mem at I
                    0x0065 => {
                        let vx = ((op & 0x0F00) >> 8) as usize;
                        let i = self.idx as usize;

                        for o in 0..vx {
                            self.reg[vx] = self.mem[i + o];
                        }
                        self.pc = self.pc + 2;
                    }
                    _ => {
                        panic!("unhandled opcode {:4X} at {:4X}", op, self.pc);
                    }
                }
            }
            // TODO: Super Chip 8
            _ => {
                panic!("unhandled opcode {:X} at {:X}", op, self.pc);
            }
        }

        if self.del_timer > 0 {
            self.del_timer = self.del_timer - 1;
        }

        if self.snd_timer > 0 {
            if self.snd_timer == 1 {
                println!("todo: beep");
            }
            self.snd_timer = self.snd_timer - 1;
        }
    }
}

impl Debug for Cpu {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "pc: {:X} op: {:X} sp {:X}", self.pc, self.op, self.sp)
    }
}
