use std::{cell::RefCell, rc::Rc, sync::mpsc};

use clap::Parser;

use lark_vm::{
    cli,
    cpu::{self, interrupts::Interrupt, Cpu, LogMsg, MemBlock, MemRw, Memory, Signal},
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
    let (logger_tx, logger_rx) = mpsc::channel();
    let (interrupt_tx, interrupt_rx) = mpsc::channel();

    let mut cpu = Cpu::new(rom, vtty.clone(), logger_tx, interrupt_rx)
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

    loop {
        if let Err(e) = cpu.step() {
            cpu.log(LogMsg::Error(format!("{:?}", e)));
        }

        for signal in logger_rx.try_iter() {
            match signal {
                Signal::Halt => {
                    eprintln!("Exiting...");
                    std::process::exit(0);
                }
                Signal::Log(msg) => match msg {
                    LogMsg::Error(e) => {
                        eprintln!("!!! Error: {e}");
                    }
                    LogMsg::DebugPuts { addr, value } => {
                        eprintln!(">>> DebugPuts: {addr:x} '{value}'");
                    }
                    LogMsg::MmioRead { .. } => {
                        eprintln!(">>> MMIO READ");
                    }
                    LogMsg::MmioWrite { .. } => {
                        eprintln!(">>> MMIO WRITE");
                    }
                    LogMsg::Instr { name, args, .. } => {
                        eprint!("{name}");
                        for (i, (_style, arg)) in args.iter().enumerate() {
                            if i != 0 {
                                eprint!(", ");
                            } else {
                                eprint!("\t");
                            }
                            eprint!("{arg}");
                        }
                        eprintln!();
                    }
                },
                Signal::Breakpoint => {
                    cpu.in_debug_mode = true;
                }
                Signal::IllegalInstr => {
                    interrupt_tx
                        .send(Interrupt::ILL_INSTR)
                        .expect("interrupt send to closed channel!");
                }
            }
        }
    }
}
