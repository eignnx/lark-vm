use std::{cell::RefCell, rc::Rc, sync::mpsc};

use clap::Parser;

use lark_vm::{
    cli,
    cpu::{self, Cpu, LogMsg, MemBlock, MemRw, Memory, Signal},
};

fn main() {
    let cli = cli::Cli::parse();

    let vec = std::fs::read(&cli.romfile).expect("Failed to read ROM file");
    let size = vec.len();
    let Some(rom) = MemBlock::from_vec(vec) else {
        eprintln!("ROM file is too large:");
        eprintln!(
            "\tFile `{}` requires {} bytes. ROM has only {} bytes.",
            cli.romfile.display(),
            size,
            cpu::ROM_SIZE,
        );
        std::process::exit(1);
    };

    let vtty = Rc::new(RefCell::new(MemBlock::new_zeroed()));
    let (tx, rx) = mpsc::channel();

    let mut cpu = Cpu::new(rom, vtty.clone(), tx)
        .with_start_addr(Memory::ROM_START)
        .in_debug_mode(cli.debug)
        .with_rom_src_path(cli.rom_src_path());

    if cli.print_rom {
        for i in Memory::ROM_START..Memory::ROM_START + size as u16 {
            let byte = cpu.mem.read_u8(i);
            print!("{:02X} ", byte);
            if i % 16 == 15 {
                println!();
            }
        }
        println!();
    }

    std::thread::spawn(move || {
        for signal in rx {
            match signal {
                Signal::Halt => {
                    println!("Exiting...");
                    std::process::exit(0);
                }
                Signal::Log(msg) => match msg {
                    LogMsg::Error(e) => {
                        println!("!!! Error: {e}");
                    }
                    LogMsg::DebugPuts { addr, value } => {
                        println!(">>> DebugPuts: {addr:x} '{value}'");
                    }
                    LogMsg::MmioRead { .. } => {
                        println!(">>> MMIO READ");
                    }
                    LogMsg::MmioWrite { .. } => {
                        println!(">>> MMIO WRITE");
                    }
                    LogMsg::Instr { name, args, .. } => {
                        print!("{name}");
                        for (i, (_style, arg)) in args.iter().enumerate() {
                            if i != 0 {
                                print!(", ");
                            } else {
                                print!("\t");
                            }
                            print!("{arg}");
                        }
                        println!();
                    }
                },
                Signal::Breakpoint => {
                    cpu.in_debug_mode = true;
                }
                Signal::IllegalInstr => {
                    println!("!!! Illegal Instruction, exiting...");
                    std::process::exit(1);
                }
            }
        }
    });

    cpu.run();
}
