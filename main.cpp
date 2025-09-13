#include <cstdint>
#include <fstream>
#include <iostream>
#include <sstream>
#include <vector>

class CHIP8 {
public:
    static constexpr int MEMORY_SIZE = 4096;
    static constexpr int START_ADDRESS = 0x200;
    static constexpr int FONTSET_START_ADDRESS = 0x50;
    static constexpr int GFX_WIDTH = 64;
    static constexpr int GFX_HEIGHT = 32;

private:
    std::vector<uint8_t> memory; // 4K memory
    std::vector<uint8_t> gfx; // 64x32 pixel display
    std::vector<uint16_t> stack; // Stack for subroutine calls
    std::vector<uint8_t> V; // CPU registers V0 to VF
    uint16_t I; // Index register
    uint16_t pc; // Program counter
    bool draw_flag; // Indicates if the screen needs to be redrawn
    uint8_t delay_timer; // 60hz timer
    uint8_t sound_timer; // 60hz timer

    void load_rom(const std::string& romfile)
    {
        std::ifstream file(romfile, std::ios::binary | std::ios::ate);
        if (!file) {
            std::cerr << "Failed to open ROM file: " << romfile << std::endl;
            exit(1);
        }

        size_t size = file.tellg();
        if (size > CHIP8::MEMORY_SIZE - CHIP8::START_ADDRESS) {
            std::cerr << "ROM file too large to fit in memory." << std::endl;
            exit(1);
        }
        file.seekg(0, std::ios::beg);
        file.read(reinterpret_cast<char*>(&memory[CHIP8::START_ADDRESS]), size);
    }

    void load_fontset()
    {
        static const uint8_t fontset[80] = {
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
        };
        for (size_t i = 0; i < 80; ++i) {
            memory[FONTSET_START_ADDRESS + i] = fontset[i];
        }
    }

    void draw_sprite_to_internal(size_t x, size_t y, uint8_t height)
    {
        V[0xF] = 0;

        for (uint8_t row = 0; row < height; ++row) {
            uint8_t sprite_byte = memory[I + row];
            for (uint8_t col = 0; col < 8; ++col) {
                if ((sprite_byte & (0x80 >> col)) != 0) {
                    size_t px = (x + col) % GFX_WIDTH;
                    size_t py = (y + row) % GFX_HEIGHT;
                    size_t idx = py * GFX_WIDTH + px;
                    if (gfx[idx]) {
                        V[0xF] = 1;
                    }
                    gfx[idx] ^= 1;
                }
            }
        }

        draw_flag = true;
    }

public:
    CHIP8(const std::string& romfile)
    {
        memory.resize(MEMORY_SIZE);
        gfx.resize(GFX_WIDTH * GFX_HEIGHT);
        V.resize(16);
        pc = START_ADDRESS;
        I = 0;
        draw_flag = false;
        delay_timer = 0;
        sound_timer = 0;

        load_rom(romfile);
        load_fontset();
    }

    bool time_to_draw() const { return draw_flag; }

    void draw_to_screen()
    {
        std::string screen;
        screen.reserve(GFX_HEIGHT * (GFX_WIDTH + 1));

        for (size_t y = 0; y < GFX_HEIGHT; ++y) {
            for (size_t x = 0; x < GFX_WIDTH; ++x) {
                screen += (gfx[y * GFX_WIDTH + x] ? '@' : ' ');
            }
            screen += '\n';
        }

        std::cout << "\033[2J\033[H" << screen;
        draw_flag = false;
    }

    void emulate_cycle()
    {
        // fetch opcode
        uint16_t opcode = (memory[pc] << 8) | memory[pc + 1];
        pc += 2;

        // decode and execute opcode
        switch (opcode & 0xF000) {

        case 0x0000:

            if (opcode == 0x00E0) { // Clear the display
                gfx.assign(GFX_WIDTH * GFX_HEIGHT, 0);
                draw_flag = true;
            } else if (opcode == 0x00EE) { // Return from subroutine
                pc = stack.back();
                stack.pop_back();
            } else {
                std::cerr << "Unknown opcode: " << std::hex << opcode << std::endl;
                exit(1);
            }
            break;

        case 0x1000:

            // jump to address NNN
            pc = opcode & 0x0FFF;
            break;

        case 0x2000:

            // call subroutine at NNN
            stack.push_back(pc);
            pc = opcode & 0x0FFF;
            break;

        case 0x3000:

            // opcode 0x3XNN
            // skip next instruction if VX == NN
            if (V[(opcode & 0x0F00) >> 8] == (opcode & 0x00FF)) {
                pc += 2;
            }
            break;

        case 0x4000:

            // opcode 0x4XNN
            // skip next instruction if VX != NN
            if (V[(opcode & 0x0F00) >> 8] != (opcode & 0x00FF)) {
                pc += 2;
            }
            break;

        case 0x5000:

            // opcode 0x5XY0
            // skip next instruction if VX == VY
            if (V[(opcode & 0x0F00) >> 8] == V[(opcode & 0x00F0) >> 4]) {
                pc += 2;
            }
            break;

        case 0x6000:

            // set register VX to NN
            V[(opcode & 0x0F00) >> 8] = static_cast<uint8_t>(opcode & 0x00FF);
            break;

        case 0x7000:

            // opcode 0x7XNN
            // add NN to register VX
            V[(opcode & 0x0F00) >> 8] += static_cast<uint8_t>(opcode & 0x00FF);
            break;

        case 0x8000:

            switch (opcode & 0x000F) {

            case 0x0000:

                // opcode 0x8XY0
                // set VX to VY
                V[(opcode & 0x0F00) >> 8] = V[(opcode & 0x00F0) >> 4];
                break;

            case 0x0001:

                // opcode 0x8XY1
                // set VX to VX OR VY
                V[(opcode & 0x0F00) >> 8] |= V[(opcode & 0x00F0) >> 4];
                break;

            case 0x0002:

                // opcode 0x8XY2
                // set VX to VX AND VY
                V[(opcode & 0x0F00) >> 8] &= V[(opcode & 0x00F0) >> 4];
                break;

            case 0x0003:

                // opcode 0x8XY3
                // set VX to VX XOR VY
                V[(opcode & 0x0F00) >> 8] ^= V[(opcode & 0x00F0) >> 4];
                break;

            case 0x0004:

                // opcode 0x8XY4
                // first add VY to VX
                // then set VF to 1 if that operation overflowed, else 0
                {
                    uint8_t x = (opcode & 0x0F00) >> 8;
                    uint8_t y = (opcode & 0x00F0) >> 4;
                    uint16_t overflow = V[x] + V[y];
                    V[x] += V[y];
                    V[0xF] = overflow > 0xFF;
                }
                break;

            case 0x0005:

                // opcode 0x8XY5
                // set VX to VX - VY, set VF to 0 if underflow, else 1
                {
                    uint8_t x = (opcode & 0x0F00) >> 8;
                    uint8_t y = (opcode & 0x00F0) >> 4;
                    bool overflow = V[x] >= V[y];
                    V[x] -= V[y];
                    V[0xF] = overflow;
                }
                break;

            case 0x0006:

                // opcode 0x8XY6
                // set VX to VX >> 1
                // set VF to least significant bit of VX before shift
                {
                    uint8_t x = (opcode & 0x0F00) >> 8;
                    uint8_t y = (opcode & 0x00F0) >> 4;
                    V[x] = V[y]; // Some interpreters use VY, ambiguous in spec
                    uint8_t overflow = V[x] & 0x1;
                    V[x] >>= 1;
                    V[0xF] = overflow;
                }
                break;

            case 0x0007:

                // opcode 0x8XY7
                // set VX to VY - VX, set VF to 0 if underflow, else 1
                {
                    uint8_t x = (opcode & 0x0F00) >> 8;
                    uint8_t y = (opcode & 0x00F0) >> 4;
                    bool overflow = V[y] >= V[x];
                    V[x] = V[y] - V[x];
                    V[0xF] = overflow;
                }
                break;

            case 0x000E:

                // opcode 0x8XYE
                // set VX to VX << 1
                // set VF to most significant bit of VX before shift
                {
                    uint8_t x = (opcode & 0x0F00) >> 8;
                    uint8_t y = (opcode & 0x00F0) >> 4;
                    V[x] = V[y]; // Some interpreters use VY, ambiguous in spec
                    uint8_t overflow = (V[x] & 0x80) >> 7;
                    V[x] <<= 1;
                    V[0xF] = overflow;
                }
                break;

            default:

                std::cerr << "Unknown opcode: " << std::hex << opcode << std::endl;
                exit(1);
                break;
            }

            break;

        case 0x9000:

            // opcode 0x9XY0
            // skip next instruction if VX != VY
            if (V[(opcode & 0x0F00) >> 8] != V[(opcode & 0x00F0) >> 4]) {
                pc += 2;
            }
            break;

        case 0xA000:

            // set index register I to NNN
            I = opcode & 0x0FFF;
            break;

        case 0xD000:

            // opcode 0xDXYN
            // draw sprite at coordinate (VX, VY) with height N
            draw_sprite_to_internal(V[(opcode & 0x0F00) >> 8],
                V[(opcode & 0x00F0) >> 4], opcode & 0x000F);

            break;

        default:

            std::cerr << "Unknown opcode: " << std::hex << opcode << std::endl;
            exit(1);
            break;
        }
    }
};

int main(int argc, const char* argv[])
{
    if (argc != 2) {
        std::cerr << "Usage: " << argv[0] << " <romfile>" << std::endl;
        return 1;
    }

    CHIP8 chip8(argv[1]);

    while (1) {
        chip8.emulate_cycle();
        if (chip8.time_to_draw()) {
            chip8.draw_to_screen();
        }
    }

    return 0;
}
