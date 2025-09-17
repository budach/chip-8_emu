use minifb::{Scale, Window, WindowOptions};
use rand::prelude::*;
use raw_cpuid::CpuId;
use std::env;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

const INSTR_PER_FRAME: usize = 200000000;
const FPS_TARGET: usize = 60;
const MEMORY_SIZE: usize = 4096;
const PROGRAM_START: usize = 0x200;
const FONTSET_START: usize = 0x50;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    gfx: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    screen_buffer: [u32; SCREEN_WIDTH * SCREEN_HEIGHT],
    v: [u8; 16],
    keys: [bool; 16],
    prev_keys: [bool; 16],
    stack: Vec<usize>,
    pc: usize,
    i: usize,
    delay_timer: u8,
    sound_timer: u8,
    window: Window,
    rng: ThreadRng,
}

impl Chip8 {
    fn new(filename: &str) -> Self {
        Chip8 {
            memory: Self::_init_memory(filename),
            gfx: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            screen_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            pc: PROGRAM_START,
            stack: Vec::new(),
            v: [0; 16],
            keys: [false; 16],
            prev_keys: [false; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            rng: rand::rng(),
            window: Window::new(
                "Rusty8",
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                WindowOptions {
                    scale: Scale::X16,
                    ..WindowOptions::default()
                },
            )
            .unwrap(),
        }
    }

    fn _init_memory(filename: &str) -> [u8; MEMORY_SIZE] {
        let rom_data = fs::read(filename).expect("Failed to open ROM file");

        if rom_data.len() > (MEMORY_SIZE - PROGRAM_START) {
            eprintln!("ROM file is too large to fit in memory");
            std::process::exit(1);
        }

        let mut memory = [0u8; MEMORY_SIZE];

        memory[PROGRAM_START..(PROGRAM_START + rom_data.len())].copy_from_slice(&rom_data);

        memory[FONTSET_START..(FONTSET_START + 80)].copy_from_slice(&[
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
        ]);

        memory
    }

    fn handle_input(&mut self) {
        self.prev_keys.copy_from_slice(&self.keys);

        self.keys[0x1] = self.window.is_key_down(minifb::Key::Key1);
        self.keys[0x2] = self.window.is_key_down(minifb::Key::Key2);
        self.keys[0x3] = self.window.is_key_down(minifb::Key::Key3);
        self.keys[0xC] = self.window.is_key_down(minifb::Key::Key4);

        self.keys[0x4] = self.window.is_key_down(minifb::Key::Q);
        self.keys[0x5] = self.window.is_key_down(minifb::Key::W);
        self.keys[0x6] = self.window.is_key_down(minifb::Key::E);
        self.keys[0xD] = self.window.is_key_down(minifb::Key::R);

        self.keys[0x7] = self.window.is_key_down(minifb::Key::A);
        self.keys[0x8] = self.window.is_key_down(minifb::Key::S);
        self.keys[0x9] = self.window.is_key_down(minifb::Key::D);
        self.keys[0xE] = self.window.is_key_down(minifb::Key::F);

        self.keys[0xA] = self.window.is_key_down(minifb::Key::Z);
        self.keys[0x0] = self.window.is_key_down(minifb::Key::X);
        self.keys[0xB] = self.window.is_key_down(minifb::Key::C);
        self.keys[0xF] = self.window.is_key_down(minifb::Key::V);
    }

    fn update_timers(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }

    #[inline(always)]
    fn draw_sprite(&mut self, mut x: usize, mut y: usize, n: usize) {
        self.v[0xF] = 0;
        x &= SCREEN_WIDTH - 1;
        y &= SCREEN_HEIGHT - 1;

        let max_rows = std::cmp::min(n, SCREEN_HEIGHT - y); // mostly 1
        let max_cols = std::cmp::min(8, SCREEN_WIDTH - x); // mostly 8

        if max_rows == 1 && max_cols == 8 {
            // no row loop and explicit range (0..8) for better compiler optimization
            let y_coord = y * SCREEN_WIDTH + x;
            let sprite_byte = self.memory[self.i];

            (0..8)
                .filter(|&bit| (sprite_byte >> (7 - bit)) & 1 == 1)
                .for_each(|bit| {
                    if self.v[0xF] == 0 {
                        self.v[0xF] |= self.gfx[y_coord + bit];
                    }
                    self.gfx[y_coord + bit] ^= 1;
                });
        } else {
            // as above, but not unrolled and max_cols unknown at compile time
            for row in 0..max_rows {
                let y_coord = (y + row) * SCREEN_WIDTH + x;
                let sprite_byte = self.memory[self.i + row];

                (0..max_cols)
                    .filter(|&bit| (sprite_byte >> (7 - bit)) & 1 == 1)
                    .for_each(|bit| {
                        // doing an if check here is slow
                        self.v[0xF] |= self.gfx[y_coord + bit];
                        self.gfx[y_coord + bit] ^= 1;
                    });
            }
        }
    }

    fn draw_to_screen(&mut self) {
        for (i, &pixel) in self.gfx.iter().enumerate() {
            self.screen_buffer[i] = if pixel == 0 { 0x000000 } else { 0xFFA500 };
        }

        self.window
            .update_with_buffer(&self.screen_buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }

    fn emulate_instruction(&mut self, how_many: usize) {
        for _ in 0..how_many {
            let opcode = u16::from_be_bytes([self.memory[self.pc], self.memory[self.pc + 1]]);
            self.pc += 2;

            match opcode & 0xF000 {
                // opcode 0x7XNN, add NN to register VX
                0x7000 => self.v[((opcode & 0x0F00) >> 8) as usize] += (opcode & 0x00FF) as u8,

                //opcode 0x4XNN, skip next instruction if VX != NN
                0x4000 => {
                    if self.v[((opcode & 0x0F00) >> 8) as usize] != (opcode & 0x00FF) as u8 {
                        self.pc += 2;
                    }
                }

                // opcode 0xDXYN, draw sprite at coordinate (VX, VY) with height N
                0xD000 => self.draw_sprite(
                    self.v[((opcode & 0x0F00) >> 8) as usize] as usize,
                    self.v[((opcode & 0x00F0) >> 4) as usize] as usize,
                    (opcode & 0x000F) as usize,
                ),

                // opcode 0x1NNN, jump to address NNN
                0x1000 => self.pc = (opcode & 0x0FFF) as usize,

                //opcode 0x2NNN, call subroutine at address NNN
                0x2000 => {
                    self.stack.push(self.pc);
                    self.pc = (opcode & 0x0FFF) as usize;
                }

                //opcode 0x3XNN, skip next instruction if VX == NN
                0x3000 => {
                    if self.v[((opcode & 0x0F00) >> 8) as usize] == (opcode & 0x00FF) as u8 {
                        self.pc += 2;
                    }
                }

                // opcode 0x5XY0, skip next instruction if VX == VY
                0x5000 => {
                    if self.v[((opcode & 0x0F00) >> 8) as usize]
                        == self.v[((opcode & 0x00F0) >> 4) as usize]
                    {
                        self.pc += 2;
                    }
                }

                // opcode 0x6XNN, set register VX to NN
                0x6000 => self.v[((opcode & 0x0F00) >> 8) as usize] = (opcode & 0x00FF) as u8,

                0x8000 => match opcode & 0x000F {
                    // opcode 0x8XY0, set VX to VY
                    0x0000 => {
                        self.v[((opcode & 0x0F00) >> 8) as usize] =
                            self.v[((opcode & 0x00F0) >> 4) as usize]
                    }

                    // opcode 0x8XY1, set VX to VX OR VY
                    0x0001 => {
                        self.v[((opcode & 0x0F00) >> 8) as usize] |=
                            self.v[((opcode & 0x00F0) >> 4) as usize];
                        self.v[0xF] = 0;
                    }

                    // opcode 0x8XY2, set VX to VX AND VY
                    0x0002 => {
                        self.v[((opcode & 0x0F00) >> 8) as usize] &=
                            self.v[((opcode & 0x00F0) >> 4) as usize];
                        self.v[0xF] = 0;
                    }

                    // opcode 0x8XY3, set VX to VX XOR VY
                    0x0003 => {
                        self.v[((opcode & 0x0F00) >> 8) as usize] ^=
                            self.v[((opcode & 0x00F0) >> 4) as usize];
                        self.v[0xF] = 0;
                    }

                    // opcode 0x8XY4, add VY to VX, set VF to 1 if overflow, else 0
                    0x0004 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize;
                        let y = ((opcode & 0x00F0) >> 4) as usize;
                        let (sum, overflow) = self.v[x].overflowing_add(self.v[y]);
                        self.v[x] = sum;
                        self.v[0xF] = overflow as u8;
                    }

                    // opcode 0x8XY5, subtract VY from VX, set VF to 0 if underflow, else 1
                    0x0005 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize;
                        let y = ((opcode & 0x00F0) >> 4) as usize;
                        let (diff, underflow) = self.v[x].overflowing_sub(self.v[y]);
                        self.v[x] = diff;
                        self.v[0xF] = (!underflow) as u8;
                    }

                    // opcode 0x8XY6, shift VX right by 1
                    // set VF to least significant bit of VX before shift
                    0x0006 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize;
                        let y = ((opcode & 0x00F0) >> 4) as usize;
                        self.v[x] = self.v[y];
                        let overflow = self.v[x] & 0x1;
                        self.v[x] >>= 1;
                        self.v[0xF] = overflow;
                    }

                    // opcode 0x8XY7, set VX to VY - VX
                    // set VF to 0 if underflow, else 1
                    0x0007 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize;
                        let y = ((opcode & 0x00F0) >> 4) as usize;
                        let (diff, underflow) = self.v[y].overflowing_sub(self.v[x]);
                        self.v[x] = diff;
                        self.v[0xF] = (!underflow) as u8;
                    }

                    // opcode 0x8XYE, set VX to VX << 1
                    // set VF to most significant bit of VX before shift
                    0x000E => {
                        let x = ((opcode & 0x0F00) >> 8) as usize;
                        let y = ((opcode & 0x00F0) >> 4) as usize;
                        self.v[x] = self.v[y];
                        let overflow = (self.v[x] & 0x80) >> 7;
                        self.v[x] <<= 1;
                        self.v[0xF] = overflow;
                    }

                    _ => println!("Unknown opcode: {:#04X}", opcode),
                },

                // opcode 0x9XY0, skip next instruction if VX != VY
                0x9000 => {
                    if self.v[((opcode & 0x0F00) >> 8) as usize]
                        != self.v[((opcode & 0x00F0) >> 4) as usize]
                    {
                        self.pc += 2;
                    }
                }

                0x0000 => match opcode & 0x00FF {
                    // opcode 0x00E0, clear the display
                    0x00E0 => self.gfx.fill(0),

                    // opcode 0x00EE, return from subroutine
                    0x00EE => self.pc = self.stack.pop().expect("Stack underflow"),

                    _ => println!("Unknown opcode: {:#04X}", opcode),
                },

                // opcode 0xANNN, set index register I to NNN
                0xA000 => self.i = (opcode & 0x0FFF) as usize,

                // opcode 0xBNNN, jump to address NNN + V0
                0xB000 => self.pc = (opcode & 0x0FFF) as usize + self.v[0] as usize,

                // opcode 0xCXNN, set VX to random byte AND NN
                0xC000 => {
                    self.v[((opcode & 0x0F00) >> 8) as usize] =
                        self.rng.random::<u8>() & (opcode & 0x00FF) as u8
                }

                0xE000 => match opcode & 0x00FF {
                    // opcode 0xEX9E, skip next instruction if key with value VX is pressed
                    0x009E => {
                        if self.keys[self.v[((opcode & 0x0F00) >> 8) as usize] as usize] {
                            self.pc += 2;
                        }
                    }

                    // opcode 0xEXA1, skip next instruction if key with value VX is not pressed
                    0x00A1 => {
                        if !self.keys[self.v[((opcode & 0x0F00) >> 8) as usize] as usize] {
                            self.pc += 2;
                        }
                    }

                    _ => println!("Unknown opcode: {:#04X}", opcode),
                },

                0xF000 => match opcode & 0x00FF {
                    // opcode 0xFX07, set VX to value of delay timer
                    0x0007 => self.v[((opcode & 0x0F00) >> 8) as usize] = self.delay_timer,

                    // opcode 0xFX0A, wait for a key release, store the value in VX
                    0x000A => {
                        //check if any key that is pressed in prev_keys is now released in keys
                        if let Some((i, _)) = self
                            .prev_keys
                            .iter()
                            .enumerate()
                            .find(|&(ref i, &key)| key && !self.keys[*i])
                        {
                            self.v[((opcode & 0x0F00) >> 8) as usize] = i as u8;
                        } else {
                            self.pc -= 2; // repeat this instruction
                        }
                    }

                    // opcode 0xFX15, set delay timer to VX
                    0x0015 => self.delay_timer = self.v[((opcode & 0x0F00) >> 8) as usize],

                    // opcode 0xFX18, set sound timer to VX,
                    0x0018 => self.sound_timer = self.v[((opcode & 0x0F00) >> 8) as usize],

                    // opcode 0xFX1E, add VX to I
                    0x001E => self.i += self.v[((opcode & 0x0F00) >> 8) as usize] as usize,

                    // opcode 0xFX29, set I to location of sprite for digit VX
                    0x0029 => {
                        self.i =
                            FONTSET_START + (self.v[((opcode & 0x0F00) >> 8) as usize] as usize * 5)
                    }

                    // opcode 0xFX33, store digits of VX in memory at addresses I, I+1, I+2
                    0x0033 => {
                        let value = self.v[((opcode & 0x0F00) >> 8) as usize];
                        self.memory[self.i] = value / 100;
                        self.memory[self.i + 1] = (value / 10) % 10;
                        self.memory[self.i + 2] = value % 10;
                    }

                    // opcode 0xFX55, store registers V0 to VX in memory starting at address I
                    0x0055 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize;
                        self.memory[self.i..=self.i + x].copy_from_slice(&self.v[0..=x]);
                        self.i += x + 1;
                    }

                    // opcode 0xFX65, read registers V0 to VX from memory starting at address I
                    0x0065 => {
                        let x = ((opcode & 0x0F00) >> 8) as usize;
                        self.v[0..=x].copy_from_slice(&self.memory[self.i..=self.i + x]);
                        self.i += x + 1;
                    }

                    _ => println!("Unknown opcode: {:#04X}", opcode),
                },

                _ => println!("Unknown opcode: {:#04X}", opcode),
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} <rom_file>", args[0]);
        std::process::exit(1);
    }

    let system_info = format!(
        "CPU: {}",
        CpuId::new()
            .get_processor_brand_string()
            .as_ref()
            .map_or_else(|| "n/a", |pbs| pbs.as_str())
    );

    let mut interpreter = Chip8::new(&args[1]);

    let frame_time_target: Duration = Duration::from_secs_f64(1.0 / FPS_TARGET as f64);
    let mut last_title_update = std::time::Instant::now();

    while interpreter.window.is_open() {
        let start_time = std::time::Instant::now();

        interpreter.handle_input();
        interpreter.update_timers();
        interpreter.emulate_instruction(INSTR_PER_FRAME);
        interpreter.draw_to_screen();

        let frame_time = start_time.elapsed();
        let sleep_time = frame_time_target.saturating_sub(frame_time);
        if sleep_time > Duration::ZERO {
            sleep(sleep_time);
        }

        let current_time = std::time::Instant::now();
        if current_time.duration_since(last_title_update) >= Duration::from_secs(2) {
            let real_fps = 1.0 / (frame_time + sleep_time).as_secs_f64();
            let status = format!(
                "Rusty8 | FPS: {:.2} | MIPS: {:.2} | {}",
                real_fps,
                (INSTR_PER_FRAME as f64 * real_fps) / 1000000.0,
                system_info
            );
            interpreter.window.set_title(&status);
            println!("{status}");
            last_title_update = current_time;
        }
    }
}
