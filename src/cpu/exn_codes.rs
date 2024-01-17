#![allow(dead_code)]

use yansi::Paint;

use super::regs::Reg;
use super::Cpu;

mod codes {
    pub const ILLEGAL_INSTR: u16 = 0x0000;
    pub const DEBUG_BREAKPOINT: u16 = 0x0001;
    pub const DIV_BY_ZERO: u16 = 0x0002;
    pub const DEBUG_PUTS: u16 = 0x0003;
}

impl Cpu {
    pub fn handle_exn(&self, code: u16) {
        match code {
            codes::ILLEGAL_INSTR => {
                eprintln!("Illegal instruction at pc={}: 0x{:X?}", self.pc, self.ir);
                std::process::exit(1);
            }

            codes::DEBUG_BREAKPOINT => {
                let lineno: u16 = self.regs.get(Reg::A0);
                let location = format!(
                    "romfile: {}:{}",
                    self.rom_src_path
                        .as_ref()
                        .map(|p| p.to_string_lossy())
                        .unwrap_or_else(|| "<unknown>".into()),
                    lineno
                );
                eprintln!("Breakpoint Exception: {location}");
                eprintln!("\t(at pc={})", self.pc);
                std::process::exit(0);
            }

            codes::DIV_BY_ZERO => {
                eprintln!("Division by zero at pc={}", self.pc);
                std::process::exit(1);
            }

            codes::DEBUG_PUTS => {
                let s_ptr = self.regs.get(Reg::A0);
                let s_len = self.regs.get(Reg::A1);
                let s = (0..s_len)
                    .map(|i| self.mem_read_u8(s_ptr, i))
                    .map(char::from)
                    .collect::<String>();
                println!("DEBUG_PUTS: {}", Paint::yellow(s).bold().italic());
            }

            other => unimplemented!("unimplemented exception code `0x{:X?}`", other),
        }
    }
}
