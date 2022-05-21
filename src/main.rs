use minifb::{Key, Window, WindowOptions};
use rand::Rng;
use rodio;
use std::env;
use std::fs;
use std::time::*;

fn main() {
    let mut memory: Vec<u8> = vec![0x0; 4096];

    let char_data = vec![
        0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0,
        0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
        0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0,
        0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
        0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0,
        0xF0, 0x80, 0xF0, 0x80, 0x80,
    ];
    let mut i = 16;
    for char_byte in char_data {
        memory[i] = char_byte;
        i += 1;
    }
    //memory[96] and memory[97] will be the location of i.

    //let mut registers_and_interpreter_data = &mut memory[0..0x200];
    //let prog = &memory[0x200..=0xE8F];
    //let vars_and_display = &mut memory[0xE90..];

    /* for y in 0..32 {
        for x in 0..8 {
            memory[0xE90 + x + (y * 8)] = 255;
        }
    }
    println!("{:?}", memory); */

    fn execute<'a>(
        command: u16,
        memory: &mut Vec<u8>,
        window: &mut Window,
    ) -> Result<(u16, u16), &'a str> {
        let le_bytes = command.to_be_bytes();
        let fb = le_bytes[0]; //firstByte
        let sb = le_bytes[1]; //secondByte
        if le_bytes == [0x00, 0xE0] {
            for i in 0..256 {
                memory[0xE90 + i] = 0;
            }
        };
        if le_bytes == [0x00, 0xEE] {
            println!("Return from Subroutine");
            return Ok((4, 0));
        } else if fb < 16 {
            /*0nnn*/
            let _address = command;
            //apparently this command is ignored???
            //execute at address (?)
            return Ok((0, 0));
        } else if fb < 32 && fb >= 16 {
            //1NNN
            let address = command - 0x1000;
            return Ok((1, address));
        } else if fb < 48 && fb >= 32 {
            //2NNN
            let address = command - 0x2000;
            println!("execute subroutine starting at address {}", address);
            return Ok((5, address));
        } else if fb < 64 && fb >= 48 {
            //3XNN
            let reg_index = usize::from(fb - 0x30);
            let value = sb;
            return if memory[reg_index] == value {
                Ok((2, 0))
            } else {
                Ok((0, 0))
            };
        } else if fb < 0x50 && fb >= 0x40 {
            //4XNN
            let reg_index = usize::from(fb - 0x40);
            return if memory[reg_index] != sb {
                Ok((2, 0))
            } else {
                Ok((0, 0))
            };
        } else if fb < 0x60 && fb >= 0x50 {
            //5XY0
            let x_i = usize::from(fb - 0x50);
            let y_i = usize::from(sb / 16);
            if memory[x_i] == memory[y_i] {
                return Ok((2, 0));
            }
            return Ok((0, 0));
        } else if fb < 0x70 && fb >= 0x60 {
            //6XNN
            let x_i = usize::from(fb - 0x60);
            memory[x_i] = sb;
            return Ok((0, 0));
        } else if fb < 0x80 && fb >= 0x70 {
            //7XNN
            let x_i = usize::from(fb - 0x70);
            if ((memory[x_i] as u16) + (sb as u16) > 255) {
                memory[x_i] = 0;
                return Ok((0, 0));
            }
            memory[x_i] += sb;
            return Ok((0, 0));
        } else if fb < 0x90 && fb >= 0x80 {
            //8XY0-8XYE
            let x_i = usize::from(fb - 0x80);
            let y_i = usize::from(sb / 16);
            let instruction = sb - (sb / 16 * 16);
            match instruction {
                0x0 => {
                    //8XY0
                    memory[x_i] = memory[y_i];
                    return Ok((0, 0));
                }
                0x1 => {
                    //8XY1
                    memory[x_i] = memory[x_i] | memory[y_i];
                    return Ok((0, 0));
                } //if memory[x_i] > 0 { 1 } else { if memory[y_i] > 0 { 1 } else { 0 } } },
                0x2 => {
                    //8XY2
                    memory[x_i] = memory[x_i] & memory[y_i];
                    return Ok((0, 0));
                }
                0x3 => {
                    //8XY3
                    memory[x_i] = memory[x_i] ^ memory[y_i];
                    return Ok((0, 0));
                }
                0x4 => {
                    //8XY4
                    let sum: u16 = (memory[x_i] as u16 + memory[y_i] as u16);
                    memory[0xf] = if sum > 0xff { 0 } else { 1 };
                    memory[x_i] = sum.to_be_bytes()[1];
                    return Ok((0, 0));
                }
                0x5 => {
                    //8XY5
                    if memory[y_i] > memory[x_i] {
                        memory[0xf] = 0;
                    }
                    memory[0xf] = 1;
                    memory[x_i] = if memory[x_i] > memory[y_i] {
                        memory[x_i] - memory[y_i]
                    } else {
                        0
                    };
                    return Ok((0, 0));
                }
                0x6 => {
                    //8XY6
                    let lsb = memory[y_i] & (1 << 0);
                    memory[x_i] = memory[y_i] >> 1;
                    memory[0xf] = lsb;
                    return Ok((0, 0));
                }
                0x7 => {
                    //8XY7
                    memory[0xf] = if memory[x_i] > memory[y_i] { 0 } else { 1 };
                    memory[x_i] = memory[y_i] - memory[x_i];
                    return Ok((0, 0));
                }
                0xE => {
                    //8XYE
                    let msb = memory[y_i] & (1 << 7);
                    memory[x_i] = memory[y_i] << 1;
                    memory[0xf] = msb;
                    return Ok((0, 0));
                }
                _ => Err("whoops"),
            }
        } else if fb < 0xa0 && fb >= 0x90 {
            // 9XY0
            let x_i = usize::from(fb - 0x90);
            let y_i = usize::from(sb / 16);
            if memory[x_i] != memory[y_i] {
                return Ok((2, 0));
            }
            return Ok((0, 0));
        } else if fb < 0xB0 && fb >= 0xA0 {
            //ANNN
            let address = command - 0xA000;
            let [a, b] = address.to_be_bytes();
            memory[96] = a;
            memory[97] = b;
            return Ok((0, 0));
        } else if fb < 0xC0 && fb >= 0xB0 {
            //BNNN
            let address: u16 = command - 0xB000 + u16::from(memory[0]);
            return Ok((1, address));
        } else if fb < 0xD0 && fb >= 0xC0 {
            //CXNN
            let x_i = usize::from(fb - 0xC0);
            memory[x_i] = rand::thread_rng().gen::<u8>() & sb;
            return Ok((0, 0));
        } else if fb < 0xE0 && fb >= 0xD0 {
            //DXYN
            let x_pos = memory[usize::from(fb - 0xD0)] % 0x40;
            let x_def = x_pos;
            let num_bytes = sb & 0x0F;
            let y_i = sb >> 4;
            let y_pos = memory[usize::from(y_i)] % 0x20;
            println!("x_pos is {}", x_pos);
            println!("y_i is {} and y_pos is {}", y_i, y_pos);
            println!("num bytes {}", num_bytes);
            println!(
                "sprite is at address {}",
                ((memory[96] as u16) << 8) + memory[97] as u16
            );
            let mut i = ((memory[96] as u16) << 8) | memory[97] as u16;
            let mut flipped_data = 0;
            let width = 8;
            let height = num_bytes;
            let mut y = y_pos;
            for _ in y_pos..y_pos + height {
                if y > 0x1f {
                    y = 0;
                }
                // i is the pointer. we need to increment it at the end of every byte processed
                let byte = memory[usize::from(i)];
                let mut clip_sprite = false;

                let mut x = x_pos;
                for _ in x_pos..x_pos + width {
                    if x > 0x3f {
                        x = 0;
                        break;
                    }
                    println!("x is {}", x);
                    let location = usize::from(x) / 8 + 0xE90 + usize::from(y * 8);
                    println!("Editing location {}", location);
                    let bit_index = 7 - (x % 8);
                    let byte_bit_index = 7 - ((x - x_def) % 8);
                    println!(
                        "bit index is {} and bytebitindex is {}",
                        bit_index, byte_bit_index
                    );
                    let cur_bit = memory[location] & 1 << bit_index;
                    let byte_bit = byte & 1 << byte_bit_index;
                    let comp_bit = if byte_bit == 0 { 0 } else { 1 << bit_index };
                    println!("cur bit is {} and byte bit is {}", cur_bit, byte_bit);
                    if flipped_data == 0 {
                        flipped_data = cur_bit & comp_bit;
                    }
                    println!("before {:08b}", memory[location]);
                    memory[location] = memory[location] ^ comp_bit;
                    println!("after  {:08b}", memory[location]);

                    x += 1;
                }
                i += 1;
                y += 1;
            }
            memory[0xf] = flipped_data;
            return Ok((0, 0));
        } else if fb < 0xF0 && fb >= 0xE0 {
            // EX9E / EXA1
            let address = usize::from(fb - 0xE0);
            match sb {
                0x9E => {
                    let keypress = get_input(window);
                    if keypress >= 17 {
                        return Ok((3, 0)); // try and get another input...
                    } else if keypress == memory[address] {
                        return Ok((2, 0));
                    } else {
                        return Ok((0, 0));
                    }
                }
                0xA1 => {
                    let keypress = get_input(window);
                    if keypress >= 17 {
                        return Ok((3, 0)); // try and get another input...
                    } else if keypress != memory[address] {
                        return Ok((2, 0));
                    } else {
                        return Ok((0, 0));
                    }
                }
                _ => {
                    panic!("Invalid command");
                }
            }
        } else if fb >= 0xF0 {
            // FX__
            let address = usize::from(fb - 0xF0);
            match sb {
                0x07 => {
                    memory[address] = memory[98];
                    return Ok((0, 0));
                }
                0x0A => {
                    let keypress = get_input(window);
                    if keypress >= 17 {
                        return Ok((3, 0)); // try and get another input...
                    }
                    println!("key {} was pressed.", keypress);
                    memory[address] = keypress;
                    return Ok((0, 0));
                }
                0x15 => {
                    memory[98] = memory[address];
                    return Ok((0, 0));
                }
                0x18 => {
                    memory[99] = memory[address];
                    return Ok((0, 0));
                }
                0x1E => {
                    let mut i = ((memory[96] as u16) << 8) + memory[97] as u16;
                    i += memory[address] as u16;
                    [memory[96], memory[97]] = i.to_be_bytes();
                    return Ok((0, 0));
                }
                0x29 => {
                    let location = (memory[address] * 5) + 16;
                    println!("Found at memory address {}", location);
                    (memory[96], memory[97]) = (0, location as u8);
                    println!(
                        "I was set to {}",
                        ((memory[96] as u16) << 8) + memory[97] as u16
                    );
                    return Ok((0, 0));
                }
                0x33 => {
                    let ones = memory[address] % 10;
                    let tens = memory[address] % 100;
                    let hundreds = memory[address] / 100;
                    let location = (((memory[96] as u16) << 8) + memory[97] as u16) as usize;
                    (memory[location], memory[location + 1], memory[location + 2]) =
                        (hundreds, tens, ones);
                    println!("Store the binary coded decimal equivalent of the value stored in register {} at addresses I, I+1, and I+2", address);
                    return Ok((0, 0));
                }
                0x55 => {
                    let index = ((memory[96] as u16) << 8) + memory[97] as u16;

                    for (i, inc) in (index as usize..=index as usize + address).zip(0..=address) {
                        memory[i] = memory[inc];
                    }

                    [memory[96], memory[97]] = (index + address as u16 + 1).to_be_bytes();
                    return Ok((0, 0));
                }
                0x65 => {
                    let index = ((memory[96] as u16) << 8) + memory[97] as u16;

                    for i in index..=index + (address as u16) {
                        memory[address] = memory[i as usize];
                    }
                    [memory[96], memory[97]] = (index + address as u16 + 1).to_be_bytes();
                    return Ok((0, 0));
                }
                _ => {
                    panic!("inavlid command");
                }
            }
        } else {
            return Err("Fail");
        }
    }

    fn get_input(window: &mut Window) -> u8 {
        let mut keypress = 17;
        window.get_keys().iter().for_each(|key| match key {
            Key::Key1 => {
                keypress = 1;
            }
            Key::Key2 => {
                keypress = 2;
            }
            Key::Key3 => {
                keypress = 3;
            }
            Key::Key4 => {
                keypress = 0xc;
            }
            Key::Q => {
                keypress = 4;
            }
            Key::W => {
                keypress = 5;
            }
            Key::E => {
                keypress = 6;
            }
            Key::R => {
                keypress = 0xd;
            }
            Key::A => {
                keypress = 7;
            }
            Key::S => {
                keypress = 8;
            }
            Key::D => {
                keypress = 9;
            }
            Key::F => {
                keypress = 0xe;
            }
            Key::Z => {
                keypress = 0xa;
            }
            Key::X => {
                keypress = 0;
            }
            Key::C => {
                keypress = 0xb;
            }
            Key::V => {
                keypress = 0xf;
            }
            _ => {}
        });
        return keypress as u8;
    }

    fn run(memory: &mut Vec<u8>) {
        let WIDTH = 128;
        let HEIGHT = 64;

        let mut window_options: WindowOptions = WindowOptions {
            borderless: false,
            title: true,
            resize: false,
            scale: minifb::Scale::X4,
            scale_mode: minifb::ScaleMode::Stretch,
            topmost: false,
            transparency: false,
            none: false,
        };
        let mut window = Window::new("Super-8", WIDTH, HEIGHT, window_options).unwrap();
        window.limit_update_rate(Some(std::time::Duration::from_micros(16667)));
        redraw(memory, &mut window, WIDTH, HEIGHT);

        let mut start_time = Instant::now();
        let frequency = rodio::source::SineWave::new(440.0);
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        sink.set_volume(0.05);
        sink.append(frequency);

        let mut last = vec![0; 256];
        let mut graphics_buffer = vec![0; WIDTH * HEIGHT];
        last.copy_from_slice(&memory[0xE90..0xE90 + 256]);

        let mut stack_pointer: usize = 0;
        let mut stack: [usize; 16] = [0; 16];
        stack[0] = 0x200;

        while window.is_open() {
            if start_time.elapsed().as_micros() >= 16667 {
                start_time = Instant::now();
                if memory[98] > 0 {
                    memory[98] -= 1;
                } //delay timer
                if memory[99] > 0 {
                    memory[99] -= 1;
                } //sound timer

                match execute(
                    ((memory[stack[stack_pointer]] as u16) << 8)
                        + memory[stack[stack_pointer] + 1] as u16,
                    memory,
                    &mut window,
                ) {
                    Ok(code) => {
                        if code.0 == 1 {
                            stack[stack_pointer] = code.1 as usize - 2;
                        } else if code.0 == 2 {
                            stack[stack_pointer] += 2; // skip next line
                        } else if code.0 == 3 {
                            stack[stack_pointer] -= 2; // redo last command (input)
                        } else if code.0 == 4 {
                            // return from sub
                            stack_pointer -= 1;
                        } else if code.0 == 5 {
                            // execute sub
                            stack_pointer += 1;
                            stack[stack_pointer] = code.1 as usize - 2;
                        }
                        stack[stack_pointer] += 2;
                    }
                    Err(err) => {
                        panic!("{}", err);
                    }
                }

                if stack[stack_pointer] >= 0xE90 {
                    stack[stack_pointer] = 0x200;
                }
                if memory[99] > 0 {
                    sink.play();
                } else {
                    sink.pause();
                }
            }
            let mut new = vec![0; 256];
            new.copy_from_slice(&memory[0xE90..0xE90 + 256]);
            if new != last {
                update_buffer(&mut new, &mut graphics_buffer, WIDTH, HEIGHT);
            }
            last.copy_from_slice(&new);
            match window.update_with_buffer(&graphics_buffer, WIDTH, HEIGHT) {
                Ok(l) => l,
                Err(error) => {
                    dbg!(error);
                }
            };
        }
    }

    fn update_buffer(memory: &mut Vec<u8>, buffer: &mut Vec<u32>, width: usize, height: usize) {
        if height < 32 || height % 32 != 0 || width < 64 || width % 64 != 0 {
            panic!("Width and height need to be greater than/equal to and a multiple of 64 and 32, respectively.")
        }
        let x_mul = width / 64;
        let y_mul = height / 32;
        let pixel_on: u32 = 0xFF32FF;
        let pixel_off: u32 = 0;

        for y in 0..32 {
            for x in 0..64 {
                let val = if memory[(x / 8) + (y * 8)] & (1 << (7 - x % 8)) != 0 {
                    1
                } else {
                    0
                }; // is the pixel supposed to be on or off?
                for new_y in (y * y_mul)..((y + 1) * y_mul) {
                    for new_x in (x * x_mul)..((x + 1) * x_mul) {
                        buffer[new_y * width + new_x] = if val == 1 { pixel_on } else { pixel_off };
                    }
                }
            }
        }
    }

    fn redraw(memory: &mut Vec<u8>, window: &mut Window, width: usize, height: usize) {
        let mut new_graphics: Vec<u32> = vec![0xFFF3FF; width * height];
        update_buffer(memory, &mut new_graphics, width, height);

        match window.update_with_buffer(&new_graphics, width, height) {
            Ok(_) => (),
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    /* fn _redraw_shitty(memory: &mut Vec<u8>, window: &mut Window) {
        let mut new_graphics: Vec<u32> = vec![0x0; 1024 * 512];
        for y in 0..32 {
            for x in 0..64 {
                let val = if memory[0xE90 + x / 8 + (y * 8)] & 1 << x % 8 != 0 {
                    0
                } else {
                    1
                };
                for new_y in (y * 16)..((y + 1) * 16) {
                    for new_x in (x * 16)..((x + 1) * 16) {
                        if new_graphics[new_y * 1024 + new_x] > 0 {
                            new_graphics[new_y * 1024 + new_x] = 0;
                        };
                        for _ in 0..24 {
                            new_graphics[new_y * 1024 + new_x] <<= val;
                        }
                    }
                }
            }
        }
        match window.update_with_buffer(&new_graphics, 1024, 512) {
            Ok(res) => {
                dbg!(res);
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    } */

    fn load_prog_into_memory(filename: &str, memory: &mut Vec<u8>) {
        let contents = fs::read(filename);
        let mut index = 0x200;
        match contents {
            Ok(content) => {
                for byte in content {
                    memory[index] = byte;
                    //println!("Set {} to 0x{:x?}", index, byte);
                    index += 1;
                }
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }

    let args: Vec<String> = env::args().collect();
    load_prog_into_memory(&args[1], &mut memory);
    run(&mut memory);
}
