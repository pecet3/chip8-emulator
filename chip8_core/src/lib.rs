use rand::random;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
pub struct Emulator {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_registers: [u8; NUM_REGISTERS],
    i_register: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    d_timer: u8,
    s_timer: u8,
}

impl Emulator {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_registers: [0; NUM_REGISTERS],
            i_register: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            d_timer: 0,
            s_timer: 0,
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }
    pub fn keypress(&mut self, idx: usize, is_pressed: bool) {
        if idx >= 16 {
            return;
        }
        self.keys[idx] = is_pressed;
    }
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();

        self.ram[start..end].copy_from_slice(data);
    }
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_registers = [0; NUM_REGISTERS];
        self.i_register = 0;

        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.d_timer = 0;
        self.s_timer = 0;
    }
    pub fn timers_tick(&mut self) {
        if self.d_timer > 0 {
            self.d_timer -= 1;
        }
        if self.s_timer > 0 {
            if self.s_timer == 1 {
                //here beep
            }
            self.s_timer -= 1;
        }
    }
    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }
    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.pc += 1;
    }
    fn pop(&mut self) -> u16 {
        self.pc -= 1;
        self.stack[self.pc as usize]
    }
    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // load v0 vx
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_register as usize;

                for idx in 0..=x {
                    self.v_registers[idx] = self.v_registers[idx + i];
                }
            }
            // store v0 vx
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_register as usize;

                for idx in 0..=x {
                    self.ram[i + idx] = self.v_registers[idx];
                }
            }
            // BCD
            (0xf, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_registers[x] as f32;

                let hundreds = (vx / 100.).floor() as u8;

                let tens = ((vx / 10.) % 10.).floor() as u8;

                let ones = (vx % 10.) as u8;

                self.ram[self.i_register as usize] = hundreds;
                self.ram[(self.i_register + 1) as usize] = tens;

                self.ram[(self.i_register + 2) as usize] = ones;
            }
            // set reg I to font address which is on the beggining on the RAM
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_registers[x] as u16;

                self.i_register = c & 5;
            }
            // I += VX
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_registers[x] as u16;

                self.i_register = self.i_register.wrapping_add(vx);
            }
            //  s timer = reg[x]
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.s_timer = self.v_registers[x];
            }
            //  delay timer = reg[x]
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.d_timer = self.v_registers[x];
            }
            // wait for any key pressed
            (0xF, _, _, 0xA) => {
                let x = digit2 as usize;
                let mut is_pressed = false;

                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_registers[x] = i as u8;
                        is_pressed = true;
                        break;
                    }
                }
                if !is_pressed {
                    self.pc -= 2;
                }
            }
            // reg[x] = delay timer
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_registers[x] = self.d_timer;
            }
            //skip if some key not pressed
            (0xE, _, 0xA, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_registers[x];
                let key = self.keys[vx as usize];

                if !key {
                    self.pc += 2;
                }
            }
            //skip if some key pressed
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_registers[x];
                let key = self.keys[vx as usize];

                if key {
                    self.pc += 2;
                }
            }
            // draw a sprite
            (0xD, _, _, _) => {
                let x_coord = self.v_registers[digit2 as usize] as u16;
                let y_coord = self.v_registers[digit3 as usize] as u16;
                let num_rows = digit4;

                let mut flipped = false;

                for y_line in 0..num_rows {
                    let addr = self.i_register + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;

                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_registers[0xF] = 1;
                } else {
                    self.v_registers[0xF] = 1;
                }
            }
            // set reg[x] to random
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();

                self.v_registers[x] = rng & nn;
            }
            // jump to v0 + nnn
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_registers[0] as u16) + nnn;
            }
            //set special reg to addr pointer
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_register = nnn;
            }
            // skip if reg[x] != reg[y]
            (9, _, _, _) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_registers[x] != self.v_registers[y] {
                    self.pc += 2
                }
            }
            // reg[x] <<=1
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_registers[x] >> 7) & 1;

                self.v_registers[x] <<= 1;
                self.v_registers[0xF] = msb;
            }
            // reg[y] -= reg[x]
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_registers[y].overflowing_sub(self.v_registers[x]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v_registers[x] = new_vx;

                // setting the last v_reg (16) because it acts as carry flag
                self.v_registers[0xF] = new_vf;
            }
            // reg[x] >>=1
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.v_registers[x] & 1;

                self.v_registers[x] >>= 1;
                self.v_registers[0xF] = lsb;
            }
            // reg[x] -= reg[y]
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_registers[x].overflowing_sub(self.v_registers[y]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v_registers[x] = new_vx;

                // setting the last v_reg (16) because it acts as carry flag
                self.v_registers[0xF] = new_vf;
            }
            // reg[x] += reg[y]
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_registers[x].overflowing_add(self.v_registers[y]);
                let new_vf = if carry { 1 } else { 0 };
                self.v_registers[x] = new_vx;

                // setting the last v_reg (16) because it acts as carry flag
                self.v_registers[0xF] = new_vf;
            }
            // reg[x] ^= reg[y]
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] ^= self.v_registers[y];
            }
            // reg[x] &= reg[y]
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] &= self.v_registers[y];
            }
            // reg[x] |= reg[y]
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] |= self.v_registers[y];
            }
            // reg[x] == reg[y]
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] = self.v_registers[y];
            }
            // reg[x] += nn
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_registers[x] = self.v_registers[x].wrapping_add(nn);
            }
            // set a register x to nn
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_registers[x] = nn;
            }
            // skip if r[x] == r[y]
            (5, _, _, _) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_registers[x] == self.v_registers[y] {
                    self.pc += 2;
                }
            }
            // skip if r[x] != nn
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_registers[x] != nn {
                    self.pc += 2;
                }
            }
            // skip if r[x] == nn
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_registers[x] == nn {
                    self.pc += 2;
                }
            }
            // call a subroutine
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            //jump to nnn
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }
            // return from subroutine
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }
            //clear screen
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            // no function
            (0, 0, 0, 0) => return,
            (_, _, _, _) => unimplemented!("There is no such opcode!: {}", op),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exe() {
        let mut emu = Emulator::new();
        emu.execute(0x00E0);
        assert_eq!(emu.pc, 1);
    }
}
