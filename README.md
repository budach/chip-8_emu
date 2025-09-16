# Basic CHIP-8 Emulator in Python and Rust

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


Test ROMs are from <https://github.com/Timendus/chip8-test-suite>.

Game ROMS are from <https://github.com/JohnEarnest/chip8Archive>.