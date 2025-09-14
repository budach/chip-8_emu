import sys
import time
from random import randint

import pygame
from cpuinfo import get_cpu_info


class C8Interpreter:

    def __init__(self, rom_file):
        self.running = True
        self.memory = [0] * 4096  # 4KB memory
        self.V = [0] * 16  # registers
        self.I = 0  # index register
        self.pc = 0x200  # program counter starts at 0x200 in memory
        self.gfx = [0] * (64 * 32)  # graphics
        self.draw_flag = False
        self.delay_timer = 0  # 60Hz timer, max 255
        self.sound_timer = 0  # 60Hz timer, max 255
        self.stack = []  # stack for subroutine calls
        self.keys = [0] * 16  # keypad with 16 keys
        self.prev_keys = [0] * 16  # previous frame key states

        # key mapping for Chip-8 keys
        self.mapping = {
            pygame.K_1: 0x1,
            pygame.K_2: 0x2,
            pygame.K_3: 0x3,
            pygame.K_4: 0xC,
            pygame.K_q: 0x4,
            pygame.K_w: 0x5,
            pygame.K_e: 0x6,
            pygame.K_r: 0xD,
            pygame.K_a: 0x7,
            pygame.K_s: 0x8,
            pygame.K_d: 0x9,
            pygame.K_f: 0xE,
            pygame.K_z: 0xA,
            pygame.K_x: 0x0,
            pygame.K_c: 0xB,
            pygame.K_v: 0xF,
        }

        # pygame setup
        pygame.init()
        self.scale = 24
        self.screen = pygame.display.set_mode((64 * self.scale, 32 * self.scale))

        # prepare memory
        self._load_rom(rom_file)
        self._load_fontset()

    def __del__(self):
        pygame.quit()

    def _load_rom(self, rom_file):
        with open(rom_file, "rb") as f:
            rom = f.read()

        if len(rom) > len(self.memory) - 0x200:
            raise ValueError("ROM too large to fit in memory")

        # load ROM into memory starting at 0x200
        for i in range(len(rom)):
            self.memory[0x200 + i] = rom[i]

    def _load_fontset(self):
        fontset = {
            "0": (0xF0, 0x90, 0x90, 0x90, 0xF0),
            "1": (0x20, 0x60, 0x20, 0x20, 0x70),
            "2": (0xF0, 0x10, 0xF0, 0x80, 0xF0),
            "3": (0xF0, 0x10, 0xF0, 0x10, 0xF0),
            "4": (0x90, 0x90, 0xF0, 0x10, 0x10),
            "5": (0xF0, 0x80, 0xF0, 0x10, 0xF0),
            "6": (0xF0, 0x80, 0xF0, 0x90, 0xF0),
            "7": (0xF0, 0x10, 0x20, 0x40, 0x40),
            "8": (0xF0, 0x90, 0xF0, 0x90, 0xF0),
            "9": (0xF0, 0x90, 0xF0, 0x10, 0xF0),
            "A": (0xF0, 0x90, 0xF0, 0x90, 0x90),
            "B": (0xE0, 0x90, 0xE0, 0x90, 0xE0),
            "C": (0xF0, 0x80, 0x80, 0x80, 0xF0),
            "D": (0xE0, 0x90, 0x90, 0x90, 0xE0),
            "E": (0xF0, 0x80, 0xF0, 0x80, 0xF0),
            "F": (0xF0, 0x80, 0xF0, 0x80, 0x80),
        }

        # load fontset into memory starting at 0x50
        for i, char_array in enumerate(fontset.values()):
            self.memory[0x50 + i * 5 + 0] = char_array[0]
            self.memory[0x50 + i * 5 + 1] = char_array[1]
            self.memory[0x50 + i * 5 + 2] = char_array[2]
            self.memory[0x50 + i * 5 + 3] = char_array[3]
            self.memory[0x50 + i * 5 + 4] = char_array[4]

    def emulate_instruction(self):
        opcode = (self.memory[self.pc] << 8) | self.memory[self.pc + 1]
        self.pc += 2

        match opcode & 0xF000:

            case 0x0000:
                if opcode == 0x00E0:
                    # opcode 0x00E0
                    # clear the display
                    self.gfx = [0] * (64 * 32)
                    self.draw_flag = True
                elif opcode == 0x00EE:
                    # opcode 0x00EE
                    # return from subroutine
                    self.pc = self.stack.pop()
                else:
                    print(f"Unknown opcode: {opcode:04X}")

            case 0x1000:
                # opcode 0x1NNN
                # jump to address NNN
                self.pc = opcode & 0x0FFF

            case 0x2000:
                # opcode 0x2NNN
                # call subroutine at address NNN
                self.stack.append(self.pc)
                self.pc = opcode & 0x0FFF

            case 0x3000:
                # opcode 0x3XNN
                # skip next instruction if VX == NN
                if self.V[(opcode & 0x0F00) >> 8] == (opcode & 0x00FF):
                    self.pc += 2

            case 0x4000:
                # opcode 0x4XNN
                # skip next instruction if VX != NN
                if self.V[(opcode & 0x0F00) >> 8] != (opcode & 0x00FF):
                    self.pc += 2

            case 0x5000:
                # opcode 0x5XY0
                # skip next instruction if VX == VY
                if self.V[(opcode & 0x0F00) >> 8] == self.V[(opcode & 0x00F0) >> 4]:
                    self.pc += 2

            case 0x6000:
                # opcode 0x6XNN
                # set register VX to NN
                self.V[(opcode & 0x0F00) >> 8] = opcode & 0x00FF

            case 0x7000:
                # opcode 0x7XNN
                # add NN to register VX
                x = (opcode & 0x0F00) >> 8
                self.V[x] = (self.V[x] + (opcode & 0x00FF)) & 255

            case 0x8000:

                match opcode & 0x000F:

                    case 0x0000:
                        # opcode 0x8XY0
                        # set VX to VY
                        self.V[(opcode & 0x0F00) >> 8] = self.V[(opcode & 0x00F0) >> 4]

                    case 0x0001:
                        # opcode 0x8XY1
                        # set VX to VX OR VY
                        self.V[(opcode & 0x0F00) >> 8] |= self.V[(opcode & 0x00F0) >> 4]
                        self.V[0xF] = 0

                    case 0x0002:
                        # opcode 0x8XY2
                        # set VX to VX AND VY
                        self.V[(opcode & 0x0F00) >> 8] &= self.V[(opcode & 0x00F0) >> 4]
                        self.V[0xF] = 0

                    case 0x0003:
                        # opcode 0x8XY3
                        # set VX to VX XOR VY
                        self.V[(opcode & 0x0F00) >> 8] ^= self.V[(opcode & 0x00F0) >> 4]
                        self.V[0xF] = 0

                    case 0x0004:
                        # opcode 0x8XY4
                        # add VY to VX, set VF to 1 if overflow, else 0
                        x = (opcode & 0x0F00) >> 8
                        y = (opcode & 0x00F0) >> 4
                        overflow = (self.V[x] + self.V[y]) > 255
                        self.V[x] = (self.V[x] + self.V[y]) & 255
                        self.V[0xF] = overflow

                    case 0x0005:
                        # opcode 0x8XY5
                        # set VX to VX - VY, set VF to 0 if underflow, else 1
                        x = (opcode & 0x0F00) >> 8
                        y = (opcode & 0x00F0) >> 4
                        overflow = self.V[x] >= self.V[y]
                        self.V[x] = (self.V[x] - self.V[y]) & 255
                        self.V[0xF] = overflow

                    case 0x0006:
                        # opcode 0x8XY6
                        # set VX to VX >> 1
                        # set VF to least significant bit of VX before shift
                        x = (opcode & 0x0F00) >> 8
                        y = (opcode & 0x00F0) >> 4
                        self.V[x] = self.V[y]
                        overflow = self.V[x] & 0x01
                        self.V[x] = (self.V[x] >> 1) & 255
                        self.V[0xF] = overflow

                    case 0x0007:
                        # opcode 0x8XY7
                        # set VX to VY - VX, set VF to 0 if underflow, else 1
                        x = (opcode & 0x0F00) >> 8
                        y = (opcode & 0x00F0) >> 4
                        overflow = self.V[y] >= self.V[x]
                        self.V[x] = (self.V[y] - self.V[x]) & 255
                        self.V[0xF] = overflow

                    case 0x000E:
                        # opcode 0x8XYE
                        # set VX to VX << 1
                        # set VF to most significant bit of VX before shift
                        x = (opcode & 0x0F00) >> 8
                        y = (opcode & 0x00F0) >> 4
                        self.V[x] = self.V[y]
                        overflow = (self.V[x] & 0x80) >> 7
                        self.V[x] = (self.V[x] << 1) & 255
                        self.V[0xF] = overflow

                    case _:
                        print(f"Unknown opcode: {opcode:04X}")

            case 0x9000:
                # opcode 0x9XY0
                # skip next instruction if VX != VY
                if self.V[(opcode & 0x0F00) >> 8] != self.V[(opcode & 0x00F0) >> 4]:
                    self.pc += 2

            case 0xA000:
                # opcode 0xANNN
                # set index register I to NNN
                self.I = opcode & 0x0FFF

            case 0xB000:
                # opcode 0xBNNN
                # jump to address NNN + V0
                self.pc = (opcode & 0x0FFF) + self.V[0]

            case 0xC000:
                # opcode 0xCXNN
                # set VX to random byte AND NN
                self.V[(opcode & 0x0F00) >> 8] = randint(0, 255) & opcode & 0x00FF

            case 0xD000:
                # opcode 0xDXYN
                # draw sprite at coordinate (VX, VY) with height N
                self._draw_sprite_to_internal(
                    x=self.V[(opcode & 0x0F00) >> 8],
                    y=self.V[(opcode & 0x00F0) >> 4],
                    n=opcode & 0x000F,
                )

            case 0xE000:

                match opcode & 0x00FF:

                    case 0x009E:
                        # opcode 0xEX9E
                        # skip next instruction if key with value VX is pressed
                        if self.keys[self.V[(opcode & 0x0F00) >> 8]]:
                            self.pc += 2

                    case 0x00A1:
                        # opcode 0xEXA1
                        # skip next instruction if key with value VX is not pressed
                        if not self.keys[self.V[(opcode & 0x0F00) >> 8]]:
                            self.pc += 2

                    case _:
                        print(f"Unknown opcode: {opcode:04X}")

            case 0xF000:

                match opcode & 0x00FF:

                    case 0x0007:
                        # opcode 0xFX07
                        # set VX to value of delay timer
                        self.V[(opcode & 0x0F00) >> 8] = self.delay_timer

                    case 0x000A:
                        # opcode 0xFX0A
                        # wait for a key release, store the value in VX
                        key_pressed = False
                        for i in range(16):
                            if self.prev_keys[i] == 1 and not self.keys[i]:
                                self.V[(opcode & 0x0F00) >> 8] = i
                                key_pressed = True
                                break
                        if not key_pressed:
                            self.pc -= 2

                    case 0x0015:
                        # opcode 0xFX15
                        # set delay timer to VX
                        self.delay_timer = self.V[(opcode & 0x0F00) >> 8]

                    case 0x0018:
                        # opcode 0xFX18
                        # set sound timer to VX
                        self.sound_timer = self.V[(opcode & 0x0F00) >> 8]

                    case 0x001E:
                        # opcode 0xFX1E
                        # add VX to I
                        x = (opcode & 0x0F00) >> 8
                        self.I = (self.I + self.V[x]) & 0x0FFF

                    case 0x0029:
                        # opcode 0xFX29
                        # set I to the location of the sprite for digit VX
                        vx = self.V[(opcode & 0x0F00) >> 8]
                        self.I = 0x50 + vx * 5  # fontset starts at 0x50

                    case 0x0033:
                        # opcode 0xFX33
                        # store digits of VX in memory at addresses I, I+1, I+2
                        vx = self.V[(opcode & 0x0F00) >> 8]
                        self.memory[self.I] = vx // 100  # hundreds
                        self.memory[self.I + 1] = (vx // 10) % 10  # tens
                        self.memory[self.I + 2] = vx % 10  # ones

                    case 0x0055:
                        # opcode 0xFX55
                        # store registers V0 to VX in memory starting at address I
                        x = (opcode & 0x0F00) >> 8
                        self.memory[self.I : self.I + x + 1] = self.V[: x + 1]
                        self.I += x + 1

                    case 0x0065:
                        # opcode 0xFX65
                        # read registers V0 to VX from memory starting at address I
                        x = (opcode & 0x0F00) >> 8
                        self.V[: x + 1] = self.memory[self.I : self.I + x + 1]
                        self.I += x + 1

                    case _:
                        print(f"Unknown opcode: {opcode:04X}")

            case _:
                print(f"Unknown opcode: {opcode:04X}")

    def draw_to_screen(self):
        gfx = self.gfx
        scale = self.scale
        screen = self.screen
        square_color = (255, 165, 0)  # orange

        # clear screen
        screen.fill((0, 0, 0))

        for y in range(32):
            row_base = y * 64
            py = y * scale
            for x in range(64):
                if gfx[row_base + x]:
                    screen.fill(square_color, (x * scale, py, scale, scale))

        pygame.display.flip()
        self.draw_flag = False

    def _draw_sprite_to_internal(self, x, y, n):
        gfx = self.gfx
        mem = self.memory
        I = self.I

        collision = 0
        x %= 64
        y %= 32
        max_rows = min(n, 32 - y)
        max_cols = min(8, 64 - x)

        for row in range(max_rows):
            y_coord = y + row
            sprite_byte = mem[I + row]

            for col in range(max_cols):
                if (sprite_byte >> (7 - col)) & 1:
                    idx = x + col + (y_coord * 64)
                    collision |= gfx[idx]
                    gfx[idx] ^= 1

        self.V[0xF] = 1 if collision else 0
        self.draw_flag = True

    def handle_input(self):
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                self.running = False

        # Save current keys as previous before updating
        self.prev_keys = self.keys[:]
        self.keys = [0] * 16
        pressed = pygame.key.get_pressed()
        for key, chip_key in self.mapping.items():
            if pressed[key]:
                self.keys[chip_key] = 1

    def update_timers(self):
        if self.delay_timer > 0:
            self.delay_timer -= 1
        if self.sound_timer > 0:
            self.sound_timer -= 1


def main(rom_file, system_info):
    interpreter = C8Interpreter(rom_file)

    FPS_TARGET = 60
    FRAME_TIME_TARGET = 1 / FPS_TARGET
    INSTR_PER_FRAME = 11  # 11 is a good default

    last_title_update = time.time()

    while interpreter.running:
        start_time = time.time()

        interpreter.handle_input()
        interpreter.update_timers()

        for _ in range(INSTR_PER_FRAME):
            interpreter.emulate_instruction()

        if interpreter.draw_flag:
            interpreter.draw_to_screen()

        frame_time = time.time() - start_time
        sleep_time = max(0, FRAME_TIME_TARGET - frame_time)

        time.sleep(sleep_time)

        current_time = time.time()
        if current_time - last_title_update >= 2.0:
            real_fps = 1 / (frame_time + sleep_time)
            pygame.display.set_caption(
                "{} | IPF: {} | FPS: {:.2f} | MIPS: {:.2f}".format(
                    system_info,
                    INSTR_PER_FRAME,
                    real_fps,
                    (INSTR_PER_FRAME * real_fps) / 1000000,
                )
            )
            last_title_update = current_time


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python main.py <ROM>")
        sys.exit(1)

    system_info = "Python: {} | CPU: {}".format(
        sys.version.split()[0],
        get_cpu_info().get("brand_raw", "Unknown CPU"),
    )

    main(sys.argv[1], system_info)
