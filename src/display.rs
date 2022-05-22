use minifb::{Key, Window, WindowOptions};
pub struct DisplayAndInput {
    pub window: Window,
    width: usize,
    height: usize,
    buffer: Vec<u32>,
    x_mul: usize,
    y_mul: usize,
    pixel_on: u32,
    keypress: u8,
    last_pixel_data: Vec<u8>,
}

impl DisplayAndInput {
    pub fn new(width: usize, height: usize, scale: Option<minifb::Scale>) -> DisplayAndInput {
        let mut window_options: WindowOptions = WindowOptions {
            borderless: false,
            title: true,
            resize: false,
            scale: scale.unwrap_or(minifb::Scale::X16),
            scale_mode: minifb::ScaleMode::Stretch,
            topmost: false,
            transparency: false,
            none: false,
        };
        DisplayAndInput {
            window: Window::new("Super-8", width, height, window_options).unwrap(),
            height,
            width,
            buffer: vec![0; width * height],
            x_mul: width / 64,
            y_mul: height / 32,
            pixel_on: 0xFF32FF,
            keypress: 17,
            last_pixel_data: vec![0; 256],
        }
    }

    pub fn refresh_and_get_input(self: &mut Self, pixel_data: &[u8]) -> u8 {
        if pixel_data != self.last_pixel_data {
            self.update_buffer(pixel_data);
        }
        self.refresh();

        self.last_pixel_data.copy_from_slice(pixel_data);

        self.get_input()
    }

    fn update_buffer(self: &mut Self, pixel_data: &[u8]) {
        if self.height < 32 || self.height % 32 != 0 || self.width < 64 || self.width % 64 != 0 {
            panic!("Width and height need to be greater than/equal to and a multiple of 64 and 32, respectively.");
        }

        for y in 0..32 {
            for x in 0..64 {
                let val = if pixel_data[(x / 8) + (y * 8)] & (1 << (7 - x % 8)) != 0 {
                    1
                } else {
                    0
                };
                for new_y in (y * self.y_mul)..((y + 1) * self.y_mul) {
                    for new_x in (x * self.x_mul)..((x + 1) * self.x_mul) {
                        self.buffer[new_y * self.width + new_x] = self.pixel_on * val;
                    }
                }
            }
        }
    }

    fn refresh(self: &mut Self) {
        match self
            .window
            .update_with_buffer(&self.buffer, self.width, self.height)
        {
            _ => {}
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    fn get_input(self: &mut Self) -> u8 {
        self.keypress = 17;
        self.window.get_keys().iter().for_each(|key| match key {
            Key::Key1 => {
                self.keypress = 1;
            }
            Key::Key2 => {
                self.keypress = 2;
            }
            Key::Key3 => {
                self.keypress = 3;
            }
            Key::Key4 => {
                self.keypress = 0xc;
            }
            Key::Q => {
                self.keypress = 4;
            }
            Key::W => {
                self.keypress = 5;
            }
            Key::E => {
                self.keypress = 6;
            }
            Key::R => {
                self.keypress = 0xd;
            }
            Key::A => {
                self.keypress = 7;
            }
            Key::S => {
                self.keypress = 8;
            }
            Key::D => {
                self.keypress = 9;
            }
            Key::F => {
                self.keypress = 0xe;
            }
            Key::Z => {
                self.keypress = 0xa;
            }
            Key::X => {
                self.keypress = 0;
            }
            Key::C => {
                self.keypress = 0xb;
            }
            Key::V => {
                self.keypress = 0xf;
            }
            _ => {}
        });
        return self.keypress;
    }
}
