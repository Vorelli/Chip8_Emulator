use std::env;
use std::thread;
use std::time::*;
mod display;
use display::DisplayAndInput;
mod sound;
use sound::SoundGenerator;
mod cpu;
use cpu::Cpu;

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
    //memory[98] is the delay timer
    //memory[99] is the sound timer
    //memory[100] is the keyboard input

    //let mut registers_and_interpreter_data = &mut memory[0..0x200];
    //let prog = &memory[0x200..=0xE8F];
    //let vars_and_display = &mut memory[0xE90..];

    fn run(memory: &mut Vec<u8>) {
        let width = 64;
        let height = 32;
        let mut display = DisplayAndInput::new(width, height, Some(minifb::Scale::X16));
        let player = SoundGenerator::new();
        let mut cpu = Cpu::new();
        let args: Vec<String> = env::args().collect();
        Cpu::load_prog_into_memory(&args[1], memory);

        let mut display_timer = Instant::now();
        let mut delay_timer = Instant::now();

        while display.window.is_open() {
            if delay_timer.elapsed().as_micros() >= 16667 {
                memory[100] = display.refresh_and_get_input(&memory[0xE90..0xE90 + 256]);
                cpu.step(memory);
                cpu.step(memory);
                cpu.step(memory);
                cpu.step(memory);
                delay_timer = Instant::now();
                if memory[98] > 0 {
                    //delay timer
                    memory[98] -= 1;
                }

                if memory[99] > 0 {
                    //sound timer
                    player.play();
                    memory[99] -= 1;
                } else {
                    player.pause();
                }
            }
        }
    }

    run(&mut memory);
}
