#![allow(dead_code)]

// use clap::Parser;

// use cpu::{Cpu, MemBlock, MemRw, Memory};

mod cli;
mod cpu;
mod log;
mod tui;
mod utils;

fn main() {
    tui::App::new().run().expect("Failed to initialize TUI");

    // let cli = cli::Cli::parse();

    // let vec = std::fs::read(&cli.romfile).expect("Failed to read ROM file");
    // let size = vec.len();
    // let Some(rom) = MemBlock::from_vec(vec) else {
    //     eprintln!("ROM file is too large:");
    //     eprintln!(
    //         "\tFile `{}` requires {} bytes. ROM has only {} bytes.",
    //         cli.romfile.display(),
    //         size,
    //         cpu::ROM_SIZE,
    //     );
    //     std::process::exit(1);
    // };

    // let mut cpu = Cpu::new(rom)
    //     .with_start_addr(Memory::ROM_START)
    //     .in_debug_mode(cli.debug)
    //     .with_rom_src_path(cli.rom_src_path());

    // if cli.print_rom {
    //     for i in Memory::ROM_START..Memory::ROM_START + size as u16 {
    //         let byte = cpu.mem.read_u8(i);
    //         print!("{:02X} ", byte);
    //         if i % 16 == 15 {
    //             println!();
    //         }
    //     }
    //     println!();
    // }

    // cpu.run();
}
