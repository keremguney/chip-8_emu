use chip_8_emu::chip8::Chip8;
use chip_8_emu::render::Platform;
use sdl2::pixels::PixelFormatEnum;
use std::env;
use std::process;
use std::time::{Duration, Instant};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: {} <Scale> <Delay> <ROM>", args[0]);
        process::exit(1);
    }

    let video_scale: u32 = args[1].parse().unwrap_or_else(|_| {
        eprintln!("Error: Scale must be a positive integer.");
        process::exit(1);
    });

    let cycle_delay: u64 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Error: Delay must be a positive integer.");
        process::exit(1);
    });

    let rom_filename = &args[3];

    let sdl_context = sdl2::init()?;

    let window_width = 64 * video_scale;
    let window_height = 32 * video_scale;
    let mut platform = Platform::new(&sdl_context, "CHIP-8 Emulator", window_width, window_height)?;

    let texture_creator = platform.create_texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA8888, 64, 32)
        .map_err(|e| e.to_string())?;

    let mut chip8 = Chip8::new();
    chip8.load_rom(rom_filename).map_err(|e| e.to_string())?;

    let mut keys = [0u8; 16];

    let cpu_clock_speed = Duration::from_millis(cycle_delay);
    let timer_clock_speed = Duration::from_micros(16670);

    let mut last_cycle_time = Instant::now();
    let mut last_timer_time = Instant::now();

    loop {
        if platform.process_input(&mut keys) {
            break;
        }
        chip8.set_keys(&keys);

        let current_time = Instant::now();

        if current_time.duration_since(last_cycle_time) >= cpu_clock_speed {
            chip8.cycle();
            last_cycle_time = current_time;
        }

        if current_time.duration_since(last_timer_time) >= timer_clock_speed {
            chip8.tick_timers();
            platform.update(&mut texture, chip8.video_buffer_u8(), 256)?;
            last_timer_time = current_time;
        }

        std::thread::sleep(Duration::from_micros(100));
    }

    Ok(())
}
