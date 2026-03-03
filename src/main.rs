use chip_8_emu::chip8::Chip8;

fn main() {
    let filename = "hello";
    let mut chip8 = Chip8::new();
    let result = chip8.load_rom(&filename);
}
