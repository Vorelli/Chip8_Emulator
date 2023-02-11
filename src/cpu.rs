use rand::Rng;
use std::fs;

pub struct Cpu {
    stack: [usize; 16],
    stack_pointer: usize,
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut stack = [0; 16];
        stack[0] = 0x200;
        Cpu {
            stack,
            stack_pointer: 0,
        }
    }

    fn execute<'a>(self: &Self, command: u16, memory: &mut Vec<u8>) -> Result<(u16, u16), &'a str> {
        if command == 0x00E0 {
            //00E0
            for i in 0..256 {
                memory[0xE90 + i] = 0;
            }
        } else if command == 0x00EE {
            //00EE
            return Ok((4, 0));
        } else if command < 0x1000 {
            //0NNN
            // this command is to be ignored....
        } else if command < 0x2000 {
            //1NNN
            return Ok((1, command & 0x0fff));
        } else if command < 0x3000 {
            //2NNN
            return Ok((5, command & 0x0fff));
        } else if command < 0x4000 {
            //3XNN
            let x_index = ((command & 0x0f00) >> 8) as usize;
            let value = (command & 0x00FF) as u8;
            if memory[x_index] == value {
                return Ok((2, 0));
            };
        } else if command < 0x5000 {
            //4XNN
            let x_index = ((command & 0x0f00) >> 8) as usize;
            let value = (command & 0x00ff) as u8;
            if memory[x_index] != value {
                return Ok((2, 0));
            };
        } else if command < 0x6000 {
            //5XY0
            let x_index = ((command & 0x0f00) >> 8) as usize;
            let y_index = ((command & 0x00f0) >> 4) as usize;
            if memory[x_index] == memory[y_index] {
                return Ok((2, 0));
            };
        } else if command < 0x7000 {
            //6XNN
            let x_index = ((command & 0x0f00) >> 8) as usize;
            memory[x_index] = (command & 0x00ff) as u8;
        } else if command < 0x8000 {
            let x_index = ((command & 0x0f00) >> 8) as usize;
            let result = memory[x_index] as u16 + (command & 0x00ff);
            memory[x_index] = (result & 0x00ff) as u8;
        } else if command < 0x9000 {
            //8XY0-8XYE
            let x_index = ((command & 0x0f00) >> 8) as usize;
            let y_index = ((command & 0x00f0) >> 4) as usize;
            match command & 0x000f {
                0x0 => {
                    memory[x_index] = memory[y_index];
                }
                0x1 => {
                    memory[x_index] = memory[x_index] | memory[y_index];
                }
                0x2 => {
                    memory[x_index] = memory[x_index] & memory[y_index];
                }
                0x3 => {
                    memory[x_index] = memory[x_index] ^ memory[y_index];
                }
                0x4 => {
                    let sum = memory[x_index] as u16 + memory[y_index] as u16;
                    memory[0xf] = if sum > 0xff { 0 } else { 1 };
                    memory[x_index] = (sum & 0x00ff) as u8;
                }
                0x5 => {
                    memory[0xf] = if memory[x_index] >= memory[y_index] {
                        1
                    } else {
                        0
                    };
                    memory[x_index] = (std::num::Wrapping(memory[x_index])
                        - std::num::Wrapping(memory[y_index]))
                    .0;
                }
                0x6 => {
                    memory[0xf] = memory[y_index] & 0x01;
                    memory[x_index] = memory[y_index] >> 1;
                }
                0x7 => {
                    memory[0xf] = if memory[x_index] > memory[y_index] {
                        0
                    } else {
                        1
                    };
                    memory[x_index] = (std::num::Wrapping(memory[y_index])
                        - std::num::Wrapping(memory[x_index]))
                    .0;
                }
                0xE => {
                    memory[0xf] = memory[y_index] & 0x80;
                    memory[x_index] = memory[y_index] << 1;
                }
                _ => return Err("Whoops!"),
            }
        } else if command < 0xA000 {
            //9XY0
            let x_index = ((command & 0xf00) >> 8) as usize;
            let y_index = ((command & 0x0f0) >> 4) as usize;
            if memory[x_index] == memory[y_index] {
                return Ok((2, 0));
            }
        } else if command < 0xB000 {
            //ANNN
            [memory[96], memory[97]] = (command & 0x0FFF).to_be_bytes();
        } else if command < 0xC000 {
            //BNNN
            let location = command & 0x0fff + (memory[0] as u16);
            return Ok((1, location));
        } else if command < 0xD000 {
            //CXNN
            let x_index = ((command & 0x0f00) >> 8) as usize;
            memory[x_index] = rand::thread_rng().gen::<u8>() & ((command & 0x00ff) as u8);
        } else if command < 0xE000 {
            //DXYN
            let width = 8;
            let height = (command & 0x000f) as usize;

            let x_index = ((command & 0x0F00) >> 8) as usize;
            let y_index = ((command & 0x00F0) >> 4) as usize;

            let x_coord = memory[x_index] as usize;
            let y_coord = memory[y_index] as usize;
            let i = ((memory[96] as usize) << 8) + (memory[97] as usize);
            memory[0xf] = 0;

            for sprite_y in 0..height {
                let sprite_byte = memory[i + sprite_y];

                for sprite_x in 0..width {
                    let x = (x_coord + sprite_x) % 64;
                    let y = (y_coord + sprite_y) % 32;

                    let sprite_pixel = (sprite_byte & (1 << 7 - sprite_x)) != 0;
                    let mem_pixel = (memory[0xE90 + (x / 8) + (y * 8)] & (1 << (7 - (x % 8)))) != 0;
                    if sprite_pixel {
                        memory[0xf] |= if mem_pixel { 1 } else { 0 };
                        memory[0xE90 + (x / 8) + (y * 8)] ^= 1 << (7 - (x % 8));
                    }
                }
            }
        } else if command < 0xF000 {
            //EX9E / EXA1
            let x_index = ((command & 0x0f00) >> 8) as usize;
            match command & 0x00ff {
                0x9E => {
                    if memory[100] == memory[x_index] {
                        return Ok((2, 0));
                    }
                }
                0xA1 => {
                    if memory[100] != memory[x_index] {
                        return Ok((2, 0));
                    }
                }
                _ => {
                    panic!("Invalid command");
                }
            }
        } else {
            //FX__
            let x_index = ((command & 0x0F00) >> 8) as usize;
            match command & 0x00ff {
                0x07 => {
                    memory[x_index] = memory[98];
                }
                0x0A => {
                    memory[x_index] = memory[100];
                }
                0x15 => {
                    memory[98] = memory[x_index];
                }
                0x18 => {
                    memory[99] = memory[x_index];
                }
                0x1E => {
                    let mut i = ((memory[96] as u16) << 8) + memory[97] as u16;
                    i += memory[x_index] as u16;
                    [memory[96], memory[97]] = i.to_be_bytes();
                }
                0x29 => {
                    let location = (memory[x_index] * 5) + 16;
                    (memory[96], memory[97]) = (0, location);
                }
                0x33 => {
                    let ones = memory[x_index] % 10;
                    let tens = memory[x_index] % 100 / 10;
                    let hundreds = memory[x_index] / 100;
                    let location = (((memory[96] as u16) << 8) + memory[97] as u16) as usize;
                    (memory[location], memory[location + 1], memory[location + 2]) =
                        (hundreds, tens, ones);
                }
                0x55 => {
                    let index = ((memory[96] as u16) << 8) + memory[97] as u16;
                    for (i, inc) in (index as usize..=index as usize + x_index).zip(0..=x_index) {
                        memory[i] = memory[inc];
                    }

                    [memory[96], memory[97]] = (index + x_index as u16 + 1).to_be_bytes();
                }
                0x65 => {
                    let index = ((memory[96] as u16) << 8) + memory[97] as u16;

                    for i in index..=index + (x_index as u16) {
                        memory[(i - index) as usize] = memory[i as usize];
                    }
                    [memory[96], memory[97]] = (index + x_index as u16 + 1).to_be_bytes();
                }
                _ => {
                    panic!("Invalid command {}", &command)
                }
            }
        }

        Ok((0, 0))
    }

    pub fn step(self: &mut Self, memory: &mut Vec<u8>) {
        let command = ((memory[self.stack[self.stack_pointer]] as u16) << 8)
            + memory[self.stack[self.stack_pointer] + 1] as u16;
        match self.execute(command, memory) {
            Ok(code) => {
                if code.0 == 1 {
                    self.stack[self.stack_pointer] = code.1 as usize - 2;
                } else if code.0 == 2 {
                    self.stack[self.stack_pointer] += 2; // skip next line
                } else if code.0 == 3 {
                    self.stack[self.stack_pointer] -= 2; // redo last command (input)
                } else if code.0 == 4 {
                    // return from sub
                    self.stack_pointer -= 1;
                } else if code.0 == 5 {
                    // execute sub
                    self.stack_pointer += 1;
                    self.stack[self.stack_pointer] = code.1 as usize - 2;
                }
                self.stack[self.stack_pointer] += 2;
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }

    pub fn load_prog_into_memory(filename: &str, memory: &mut Vec<u8>) {
        let contents = fs::read(filename);
        let mut index = 0x200;
        match contents {
            Ok(content) => {
                for byte in content {
                    memory[index] = byte;
                    index += 1;
                }
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }
}
