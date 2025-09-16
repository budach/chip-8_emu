use minifb::{Scale, Window, WindowOptions};
use raw_cpuid::CpuId;
use std::env;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

const INSTR_PER_FRAME: usize = 11;
const FPS_TARGET: usize = 60;
const MEMORY_SIZE: usize = 4096;
const PROGRAM_START: usize = 0x200;
const FONTSET_START: usize = 0x50;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

struct Chip8 {
    memory: [u8; MEMORY_SIZE], // TODO also test Vec
    gfx: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    pc: usize,
    stack: Vec<usize>,
    v: [u8; 16],
    i: usize,
    delay_timer: u8,
    window: Window,
}

impl Chip8 {
    fn new(filename: &str) -> Self {
        Chip8 {
            memory: Self::_init_memory(filename),
            gfx: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            pc: PROGRAM_START,
            stack: Vec::new(),
            v: [0; 16],
            i: 0,
            delay_timer: 0,
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

    fn handle_input(&mut self) {}

    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    fn draw_sprite(&mut self, mut x: usize, mut y: usize, n: usize) {
        let mut collision = 0;

        x %= SCREEN_WIDTH;
        y %= SCREEN_HEIGHT;

        let max_rows = std::cmp::min(n, SCREEN_HEIGHT - y);
        let max_cols = std::cmp::min(8, SCREEN_WIDTH - x);

        for row in 0..max_rows {
            let y_coord = (y + row) * SCREEN_WIDTH + x;
            let sprite_byte = self.memory[self.i + row];

            for col in 0..max_cols {
                if (sprite_byte >> (7 - col)) & 1 == 1 {
                    collision |= self.gfx[y_coord + col];
                    self.gfx[y_coord + col] ^= 1;
                }
            }
        }

        self.v[0xF] = collision;
    }

    fn draw_to_screen(&mut self) {
        self.window
            .update_with_buffer(
                &self
                    .gfx
                    .iter()
                    .map(|&pixel| if pixel == 0 { 0x000000 } else { 0xFFA500 })
                    .collect::<Vec<u32>>(),
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
            )
            .unwrap();
    }

    fn emulate_instruction(&mut self, how_many: usize) {
        let mut opcode: u16;

        for _ in 0..how_many {
            opcode = ((self.memory[self.pc] as u16) << 8) | (self.memory[self.pc + 1] as u16);
            self.pc += 2;

            match opcode & 0xF000 {
                0x0000 => match opcode & 0x00FF {
                    // opcode 0x00E0, clear the display
                    0x00E0 => self.gfx.fill(0),

                    // opcode 0x00EE, return from subroutine
                    0x00EE => self.pc = self.stack.pop().expect("Stack underflow"),

                    _ => println!("Unknown opcode: {:#04X}", opcode),
                },

                // opcode 0x1NNN, jump to address NNN
                0x1000 => self.pc = (opcode & 0x0FFF) as usize,

                // opcode 0x6XNN, set register VX to NN
                0x6000 => self.v[((opcode & 0x0F00) >> 8) as usize] = (opcode & 0x00FF) as u8,

                // opcode 0xANNN, set index register I to NNN
                0xA000 => self.i = (opcode & 0x0FFF) as usize,

                // opcode 0xDXYN, draw sprite at coordinate (VX, VY) with height N
                0xD000 => self.draw_sprite(
                    self.v[((opcode & 0x0F00) >> 8) as usize] as usize,
                    self.v[((opcode & 0x00F0) >> 4) as usize] as usize,
                    (opcode & 0x000F) as usize,
                ),

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
        "Rusty8 | CPU: {}",
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
            last_title_update = current_time;
            let real_fps = 1.0 / (frame_time + sleep_time).as_secs_f64();
            interpreter.window.set_title(&format!(
                "{} | FPS: {:.2} | MIPS: {:.2}",
                system_info,
                real_fps,
                (INSTR_PER_FRAME as f64 * real_fps) / 1000000.0
            ));
        }
    }
}
