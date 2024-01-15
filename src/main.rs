#![allow(dead_code)]

use std::path::PathBuf;

use cpu::{Cpu, MemBlock, Memory};

use crate::cpu::MemRw;

mod cpu;
mod log;
mod utils;

fn main() {
    if cfg!(debug_assertions) {
        use yansi::Paint;

        if cfg!(windows) && !Paint::enable_windows_ascii() {
            Paint::disable();
        }
    }

    let Some(rom_path) = std::env::args().nth(1) else {
        eprintln!("usage: lark <PATH-TO-ROM-BIN>");
        std::process::exit(1);
    };

    let rom_path = PathBuf::from(rom_path);
    let rom_path_no_ext = rom_path.with_extension("");
    let src_path = rom_path_no_ext.with_extension("lark");

    let vec = std::fs::read(&rom_path).expect("Failed to read ROM file");
    let size = vec.len();
    let rom = MemBlock::from_vec(vec).expect("ROM file too large");

    let mut cpu = Cpu::new(Memory::ROM_START, rom, Some(src_path));

    if cfg!(debug_assertions) {
        for i in Memory::ROM_START..Memory::ROM_START + size as u16 {
            let byte = cpu.mem.read_u8(i);
            print!("{:02X} ", byte);
            if i % 16 == 15 {
                println!();
            }
        }
        println!();
    }

    cpu.run();
}
