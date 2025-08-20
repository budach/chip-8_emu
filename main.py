import sys

from C8Interpreter import C8Interpreter


def main(rom_file):
    interpreter = C8Interpreter(rom_file)

    while interpreter.running:
        interpreter.clock.tick(60)  # Cap at 60 FPS

        for _ in range(10):
            interpreter.emulate_cycle()

        if interpreter.draw_flag:
            interpreter.draw_to_screen()

        interpreter.handle_input()
        interpreter.update_timers()


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python main.py <ROM>")
        sys.exit(1)
    main(sys.argv[1])
