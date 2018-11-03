pub const TIME_PER_UPDATE: f64 = 1000.0 / 60.0;
const GFX_SIZE: usize = 64 * 32;
const MAX_ROM_SIZE: usize = 0x1000 - 0x200;

pub struct Cpu {
    mem: [u8; 4096],
    gfx: [u8; GFX_SIZE],
    reg: [u8; 16],
    op: u16,
    idx: u16,
    pc: usize,
    del_timer: u8,
    snd_timer: u8,
    stack: [usize; 16],
    sp: usize,
    key: [u8; 16],
    pub draw: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            mem: [0; 4096],
            gfx: [0; GFX_SIZE],
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
        }
    }

    pub fn load_rom(&mut self) {
        let rom = include_bytes!("../roms/pong.ch8");
        let size = rom.len();
        if size > MAX_ROM_SIZE {
            panic!("the ROM is too big")
        }
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
            // SYS nnn: system reserved
            0x0000 => {
                match op & 0x0FFF {
                    // CLS clear screen
                    0x00E0 => {
                        self.gfx = [0; GFX_SIZE];
                        self.draw = true;
                    }
                    // RET return from subroutine
                    0x00EE => {
                        let dst = self.stack[self.sp];
                        self.pc = dst;
                        if self.sp > 0 {
                            self.sp = self.sp - 1;
                        }
                    }
                    _ => {} // ignore SYS opcodes
                }
            }
            // JP nnn: jump to
            0x1000 => {
                let dst = op & 0x0FFF;
                self.pc = dst.into();
            }
            // CALL nnn: subroutine
            0x2000 => {
                self.stack[self.sp] = self.pc;
                self.sp = self.sp + 1;

                let dst = op & 0xFFF;
                self.pc = dst.into();
            }
            _ => {
                println!("unhandled opcode {:X}", op);
                self.pc = self.pc + 2;
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
