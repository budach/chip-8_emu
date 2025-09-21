# Basic CHIP-8 Emulator in Python, Rust and C

## Python Version

Install dependencies:
```bash
pip install -r pyslow8/requirements.txt
```

Run a game:
```bash
python pyslow8/main.py game_roms/breakout.ch8
```

## Rust Version

Build and run:
```bash
cd rusty8
cargo run --release -- ../game_roms/breakout.ch8
```

## C Version

To build the C version, make sure [Raylib](https://github.com/raysan5/raylib) is installed and its headers and libraries are accessible to _gcc_. Note: this was only tested on Windows so far. The LDFLAGS in the makefile might not work on Linux.

Build:
```bash
cd sea8
make release
```

Run a game:
```bash
sea8.exe ../game_roms/breakout.ch8
```

## Notes

Test ROMs are from <https://github.com/Timendus/chip8-test-suite>.

Game ROMS are from <https://github.com/JohnEarnest/chip8Archive>.