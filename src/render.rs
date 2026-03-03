use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

pub struct Platform {
    canvas: Canvas<Window>,
    event_pump: EventPump,
}

impl Platform {
    pub fn new(
        sdl_context: &sdl2::Sdl,
        title: &str,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, String> {
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window(title, window_width, window_height)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .accelerated()
            .build()
            .map_err(|e| e.to_string())?;

        let event_pump = sdl_context.event_pump()?;

        Ok(Self { canvas, event_pump })
    }

    pub fn create_texture_creator(&self) -> TextureCreator<WindowContext> {
        self.canvas.texture_creator()
    }

    pub fn update(
        &mut self,
        texture: &mut Texture,
        buffer: &[u8],
        pitch: usize,
    ) -> Result<(), String> {
        texture
            .update(None, buffer, pitch)
            .map_err(|e| e.to_string())?;
        self.canvas.clear();
        self.canvas.copy(texture, None, None)?;
        self.canvas.present();

        Ok(())
    }

    pub fn process_input(&mut self, keys: &mut [u8; 16]) -> bool {
        let mut quit = false;

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    quit = true;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(idx) = map_key(keycode) {
                        keys[idx] = 1;
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(idx) = map_key(keycode) {
                        keys[idx] = 0;
                    }
                }
                _ => {}
            }
        }

        quit
    }
}

fn map_key(keycode: Keycode) -> Option<usize> {
    match keycode {
        Keycode::X => Some(0x0),
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::Z => Some(0xA),
        Keycode::C => Some(0xB),
        Keycode::Num4 => Some(0xC),
        Keycode::R => Some(0xD),
        Keycode::F => Some(0xE),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
