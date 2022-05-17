extern crate rand;
use minifb::*;
use rand::Rng;
use std::env;
use std::fs;

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

    fn execute(command: u16, memory: &mut Vec<u8>, window: &minifb::Window) -> Result<char> {
        println!("{}", command);
        let le_bytes = command.to_be_bytes();
        let fb = le_bytes[0]; //firstByte
        let sb = le_bytes[1]; //secondByte
        println!("{:?}", le_bytes);
        if le_bytes == [0x00, 0xE0] {
            for y in 0..4u8 {
                for x in 0..8u8 {
                    memory[466 + usize::from(x + (y * 8))] = 0x00;
                }
            }
        };
        if le_bytes == [0x00, 0xEE] {
            println!("Return from Subroutine");
            return Ok(0 as char);
        } else if fb < 16 {
            /*0nnn*/
            let address = command;
            println!("Execute at address {}", address);
            return Ok(0 as char);
        } else if fb < 32 && fb >= 16 {
            //1NNN
            let address = command - 0x10;
            println!("jump to address {}", address);
            return Ok(0 as char);
        } else if fb < 48 && fb >= 32 {
            //2NNN
            let address = command - 0x20;
            println!("execute subroutine starting at address {}", address);
            return Ok(0 as char);
        } else if fb < 64 && fb >= 48 {
            //3XNN
            let reg_index = usize::from(fb - 0x30);
            let value = sb;
            if memory[reg_index] == value {
                println!("Skip the following instruction")
            }
            return Ok(0 as char);
        } else if fb < 0x50 && fb >= 0x40 {
            //4XNN
            let reg_index = usize::from(fb - 0x40);
            if memory[reg_index] != sb {
                println!("Skip the following instruction")
            }
            return Ok(0 as char);
        } else if fb < 0x60 && fb >= 0x50 {
            //5XY0
            let x_i = usize::from(fb - 0x50);
            let y_i = usize::from(sb / 16);
            if memory[x_i] == memory[y_i] {
                println!("skip the follwing insturction");
            }
            return Ok(0 as char);
        } else if fb < 0x70 && fb >= 0x60 {
            //6XNN
            let x_i = usize::from(fb - 0x60);
            memory[x_i] = sb;
            return Ok(0 as char);
        } else if fb < 0x80 && fb >= 0x70 {
            //7XNN
            let x_i = usize::from(fb - 0x70);
            memory[x_i] += sb;
            return Ok(0 as char);
        } else if fb < 0x90 && fb >= 0x80 {
            //8XY0-8XYE
            let x_i = usize::from(fb - 0x80);
            let y_i = usize::from(sb / 16);
            let instruction = sb - (sb / 16 * 16);
            match instruction {
                0x0 => {
                    memory[x_i] = memory[y_i];
                    return Ok(0 as char);
                }
                0x1 => {
                    memory[x_i] = memory[x_i] | memory[y_i];
                    return Ok(0 as char);
                } //if memory[x_i] > 0 { 1 } else { if memory[y_i] > 0 { 1 } else { 0 } } },
                0x2 => {
                    memory[x_i] = memory[x_i] & memory[y_i];
                    return Ok(0 as char);
                }
                0x3 => {
                    memory[x_i] = memory[x_i] ^ memory[y_i];
                    return Ok(0 as char);
                }
                0x4 => {
                    let sum: u16 = (memory[x_i] + memory[y_i]).into();
                    memory[0xf] = if sum > 0xff { 0 } else { 1 };
                    memory[x_i] = sum.to_be_bytes()[1];
                    return Ok(0 as char);
                }
                0x5 => {
                    if memory[y_i] > memory[x_i] {
                        memory[0xf] = 0;
                    }
                    memory[0xf] = 1;
                    memory[x_i] = memory[x_i] - memory[y_i];
                    return Ok(0 as char);
                }
                0x6 => {
                    let lsb = memory[y_i] & (1 << 0);
                    memory[x_i] = memory[y_i] >> 1;
                    memory[0xf] = lsb;
                    return Ok(0 as char);
                }
                0x7 => {
                    memory[0xf] = if memory[x_i] > memory[y_i] { 0 } else { 1 };
                    memory[x_i] = memory[y_i] - memory[x_i];
                    return Ok(0 as char);
                }
                0xE => {
                    let msb = memory[y_i] & (1 << 7);
                    memory[x_i] = memory[y_i] << 1;
                    memory[0xf] = msb;
                    return Ok(0 as char);
                }
                _ => {
                    return core::result::Result::Err(minifb::Error::UpdateFailed(String::from(
                        "whoops",
                    )));
                }
            }
        } else if fb < 0xa0 && fb >= 0x90 {
            // 9XY0
            let x_i = usize::from(fb - 0x90);
            let y_i = usize::from(sb / 16);
            if memory[x_i] != memory[y_i] {
                println!("Skip the following instruction");
            }
            return Ok(0 as char);
        } else if fb < 0xB0 && fb >= 0xA0 {
            //ANNN
            let address = command - 0xA000;
            let [a, b] = address.to_be_bytes();
            memory[96] = a;
            memory[97] = b;
            return Ok(0 as char);
        } else if fb < 0xC0 && fb >= 0xB0 {
            //BNNN
            let address: u16 = command - 0xB000 + u16::from(memory[0]);
            println!("jump to address {}", address);
            return Ok(0 as char);
        } else if fb < 0xD0 && fb >= 0xC0 {
            //CXNN
            let x_i = usize::from(fb - 0xC0);
            memory[x_i] = rand::thread_rng().gen::<u8>() & sb;
            return Ok(0 as char);
        } else if fb < 0xE0 && fb >= 0xD0 {
            //DXYN
            let x_pos = memory[usize::from(fb - 0xD0)] % 0x40;
            let x_def = x_pos;
            let y_i = sb / 16;
            let y_pos = memory[usize::from(y_i)] % 0x20;
            let num_bytes = sb - y_i;
            let mut i = ((memory[96] as u16) << 8) | memory[97] as u16;
            let mut flipped_data = 0;
            let width = 8;
            let height = num_bytes / 8;
            for y in y_pos..y_pos + height {
                if y > 0x1f {
                    return Ok(1 as char);
                }
                // i is the pointer. we need to increment it at the end of every byte processed
                let byte = memory[usize::from(i)];
                let mut clip_sprite = false;

                for x in x_pos..x_pos + width {
                    if x > 0x3f {
                        clip_sprite = true;
                        break;
                    } else {
                        let location = usize::from(x) / 8 + 0xE90 + usize::from(y * 8);
                        let bit_index = x % 8;
                        let byte_bit_index = x - x_def;
                        let mem_bit = memory[location] & 1 << bit_index;
                        let byte_bit = byte & 1 << byte_bit_index;
                        if flipped_data == 0 {
                            flipped_data = mem_bit & byte_bit;
                        }
                        memory[location] = memory[location] ^ byte_bit;
                    }
                }
                let w_u16 = width as u16;
                if clip_sprite {
                    i += w_u16 - (i % w_u16);
                } else {
                    i += 1;
                };
            }
            memory[0xf] = flipped_data;
            return Ok(1 as char);
        } else if fb < 0xF0 && fb >= 0xE0 {
            // EX9E / EXA1
            let address = usize::from(fb - 0xE0);
            match sb {
                0x9E => {
                    println!(
                        "if {} key is pressed, skip the following instruction",
                        memory[address]
                    );
                }
                0xA1 => {
                    println!(
                        "If {} key is not pressed, skip the following instruction",
                        memory[address]
                    );
                }
                _ => {
                    panic!("Invalid command");
                }
            }
            return Ok(0 as char);
        } else if fb >= 0xF0 {
            // FX__
            let address = usize::from(fb - 0xF0);
            match sb {
                0x07 => {
                    println!("Get the delay timer and set it in register {}", address);
                    return Ok(0 as char);
                }
                0x0A => {
                    let mut keypress = 17;
                    while (keypress > 0xf) {
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
                        })
                    }
                    memory[address] = keypress;
                    return Ok(0 as char);
                }
                0x15 => {
                    println!("Set the delay timer to the value in register {}", address);
                    return Ok(0 as char);
                }
                0x18 => {
                    println!("Set the sound timer to the value in register {}", address);
                    return Ok(0 as char);
                }
                0x1E => {
                    println!("Add the value stored in register {} to register I", address);
                    return Ok(0 as char);
                }
                0x29 => {
                    println!("Set I to the location of the specific sprite of the character specified in register {}", address);
                    return Ok(0 as char);
                }
                0x33 => {
                    println!("Store the binary coded decimal equivalent of the value stored in register {} at addresses I, I+1, and I+2", address);
                    return Ok(0 as char);
                }
                0x55 => {
                    for i in 0..=address {
                        println!("Set I to the value of register {}", i);
                        println!("Increment I");
                    }
                    println!("Increment I again");
                    return Ok(0 as char);
                }
                0x65 => {
                    for i in 0..=address {
                        println!(
                            "Set register {} to the value stored at the location of I",
                            address
                        );
                        println!("Increment I");
                    }
                    println!("Increment I again");
                    return Ok(0 as char);
                }
                _ => {
                    panic!("inavlid command");
                }
            }
        } else {
            return core::result::Result::Err(minifb::Error::UpdateFailed(String::from("Fail")));
        }
    }

    fn run(memory: &mut Vec<u8>) {
        let mut window =
            minifb::Window::new("Super-8", 1024, 512, WindowOptions::default()).unwrap();
        while window.is_open() {
            match execute(
                ((memory[0x200] as u16) << 8) + memory[0x201] as u16,
                memory,
                &window,
            ) {
                Ok(code) => {
                    if code == 1 as char {
                        redraw(memory, &mut window);
                    }
                }
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
    }

    fn redraw(memory: &mut Vec<u8>, window: &mut minifb::Window) {
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
    }

    fn load_prog_into_memory(filename: &str, memory: &mut Vec<u8>) {
        let contents = fs::read(filename);
        let mut index = 0x200;
        match contents {
            Ok(content) => {
                for byte in content {
                    memory[index] = byte;
                    println!("Set {} to 0x{:x?}", index, byte);
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
