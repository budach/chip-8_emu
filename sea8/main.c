#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "raylib.h"

#define INSTR_PER_FRAME 11
#define MEM_SIZE 4096
#define PROGRAM_START 0x200
#define FONTSET_START 0x50
#define SCREEN_WIDTH 64
#define SCREEN_HEIGHT 32
#define SCREEN_SCALE 15

struct Chip8 {
    uint8_t mem[MEM_SIZE];
    uint8_t gfx[SCREEN_WIDTH * SCREEN_HEIGHT];
    uint8_t V[16];
    uint8_t keys[16];
    uint8_t prev_keys[16];
    size_t pc;
    size_t I;
    uint8_t delay_timer;
    uint8_t sound_timer;
};

void chip8_init(struct Chip8* chip8, const char* rom_path)
{
    // read ROM file into mem

    FILE* rom = fopen(rom_path, "rb");
    if (!rom) {
        printf("Failed to open ROM file");
        exit(1);
    }

    fseek(rom, 0, SEEK_END);
    long rom_size = ftell(rom);
    fseek(rom, 0, SEEK_SET);

    if (rom_size > (MEM_SIZE - PROGRAM_START)) {
        printf("ROM file is too large to fit in mem");
        exit(1);
    }

    fread(&chip8->mem[PROGRAM_START], 1, rom_size, rom);
    fclose(rom);

    // load fontset into mem

    uint8_t fontset[80] = {
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

    memcpy(&chip8->mem[FONTSET_START], fontset, 80);

    // init other arrays

    memset(chip8->gfx, 0, sizeof(chip8->gfx));
    memset(chip8->V, 0, sizeof(chip8->V));
    memset(chip8->keys, 0, sizeof(chip8->keys));
    memset(chip8->prev_keys, 0, sizeof(chip8->prev_keys));

    // init other variables

    chip8->pc = PROGRAM_START;
    chip8->I = 0;
    chip8->delay_timer = 0;
    chip8->sound_timer = 0;
}

void chip8_handle_input(struct Chip8* chip8)
{
    // TODO
    memcpy(chip8->prev_keys, chip8->keys, sizeof(chip8->keys));
}

void chip8_update_timers(struct Chip8* chip8)
{
    if (chip8->delay_timer > 0) {
        chip8->delay_timer--;
    }
    if (chip8->sound_timer > 0) {
        chip8->sound_timer--;
    }
}

void chip8_draw_sprite(struct Chip8* c8, uint8_t x, uint8_t y, uint8_t n)
{
    c8->V[0xF] = 0;

    uint8_t max_rows = n < SCREEN_HEIGHT - y ? n : SCREEN_HEIGHT - y; // mostly 1
    uint8_t max_cols = 8 < SCREEN_WIDTH - x ? 8 : SCREEN_WIDTH - x; // mostly 8

    for (uint8_t row = 0; row < max_rows; ++row) {
        size_t y_coord = (y + row) * SCREEN_WIDTH + x;
        uint8_t sprite_byte = c8->mem[c8->I + row];

        for (uint8_t bit = 0; bit < max_cols; ++bit) {
            if ((sprite_byte & (0x80 >> bit)) != 0) {
                c8->V[0xF] |= c8->gfx[y_coord + bit];
                c8->gfx[y_coord + bit] ^= 1;
            }
        }
    }
}

void chip8_emulate_instructions(struct Chip8* c8, int instr_count)
{
    for (int i = 0; i < instr_count; i++) {
        uint16_t opcode = (c8->mem[c8->pc] << 8) | c8->mem[c8->pc + 1];
        c8->pc += 2;

        switch (opcode & 0xF000) {

        case 0x0000:

            switch (opcode & 0x00FF) {

            case 0x00E0:

                // opcode 0x00E0, clear the display
                memset(c8->gfx, 0, sizeof(c8->gfx));
                break;

            default:

                printf("Unknown opcode: 0x%04X\n", opcode);
                exit(1);
                break;
            }

            break;

        case 0x1000:

            // opcode 0x1NNN, jump to address NNN
            c8->pc = opcode & 0x0FFF;
            break;

        case 0x6000:

            // opcode 0x6XNN, set register VX to NN
            c8->V[(opcode & 0x0F00) >> 8] = opcode & 0x00FF;
            break;

        case 0xA000:

            // opcode 0xANNN, set index register I to NNN
            c8->I = opcode & 0x0FFF;
            break;

        case 0xD000:

            // opcode 0xDXYN, draw sprite at coordinate (VX, VY) with height N
            chip8_draw_sprite(
                c8,
                c8->V[(opcode & 0x0F00) >> 8] & (SCREEN_WIDTH - 1),
                c8->V[(opcode & 0x00F0) >> 4] & (SCREEN_HEIGHT - 1),
                opcode & 0x000F);
            break;

        default:
            printf("Unknown opcode: 0x%04X\n", opcode);
            exit(1);
            break;
        }
    }
}

void draw_frame_to_window(uint8_t* gfx)
{
    BeginDrawing();
    ClearBackground(BLACK);

    for (int y = 0; y < SCREEN_HEIGHT; ++y) {
        for (int x = 0; x < SCREEN_WIDTH; ++x) {
            if (gfx[y * SCREEN_WIDTH + x]) {
                DrawRectangle(x * SCREEN_SCALE, y * SCREEN_SCALE, SCREEN_SCALE, SCREEN_SCALE, BEIGE);
            }
        }
    }

    EndDrawing();
}

int main(int argc, char** argv)
{
    if (argc != 2) {
        printf("Usage: %s <rom_file>\n", argv[0]);
        return 1;
    }

    struct Chip8 c8;
    chip8_init(&c8, argv[1]);

    InitWindow(SCREEN_WIDTH * SCREEN_SCALE, SCREEN_HEIGHT * SCREEN_SCALE, "Sea8");
    SetTargetFPS(60);

    double last_status_update = GetTime();
    char status[128] = { 0 };

    while (!WindowShouldClose()) {
        chip8_handle_input(&c8);
        chip8_update_timers(&c8);
        chip8_emulate_instructions(&c8, INSTR_PER_FRAME);
        draw_frame_to_window(c8.gfx);

        double current_time = GetTime();
        if (current_time - last_status_update >= 2.0) { // every 2 seconds
            snprintf(status, sizeof(status), "Sea8 | FT: %.4fms", GetFrameTime() * 1000);
            SetWindowTitle(status);
            last_status_update = current_time;
        }
    }

    CloseWindow();

    return 0;
}