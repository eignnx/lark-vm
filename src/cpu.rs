use std::{collections::BTreeSet, path::PathBuf};

use crate::{cpu::regs::Reg, utils::s16};

use self::regs::RegisterFile;

mod debugger;
mod decode;
mod dex;
mod exn_codes;
mod opcodes;
mod regs;

pub const KIB: usize = 1024;
pub const STACK_INIT: u16 = Memory::USER_END - 1;

pub struct Cpu {
    pub regs: RegisterFile,

    /// Program counter.
    pub pc: u16,

    /// Instruction register.
    pub ir: u32,

    /// Hi and Lo registers are used for multipliation and division.
    pub hi: s16,
    pub lo: s16,

    pub mem: Memory,

    pub in_debug_mode: bool,
    pub breakpoints: BTreeSet<u16>,
    pub rom_src_path: Option<PathBuf>,
}

impl Cpu {
    pub fn new(rom: MemBlock<ROM_SIZE>) -> Self {
        Self {
            regs: RegisterFile::new(STACK_INIT),
            pc: 0,
            ir: 0,
            hi: s16::default(),
            lo: s16::default(),
            mem: Memory::new(rom),
            in_debug_mode: false,
            breakpoints: BTreeSet::new(),
            rom_src_path: None,
        }
    }

    pub fn with_start_addr(mut self, start_addr: u16) -> Self {
        self.pc = start_addr;
        self
    }

    pub fn in_debug_mode(mut self, debug: bool) -> Self {
        self.in_debug_mode = debug;
        self
    }

    pub fn with_rom_src_path(mut self, rom_src_path: PathBuf) -> Self {
        self.rom_src_path = Some(rom_src_path);
        self
    }

    pub fn run(&mut self) {
        loop {
            if self.breakpoints.contains(&self.pc) {
                self.in_debug_mode = true;
            }

            self.fetch();
            match self.decode_and_execute() {
                Ok(()) => {}
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        }
    }

    fn mem_read_s16(&self, addr_base: u16, addr_offset: i16) -> s16 {
        self.mem
            .read_s16(self.mem.compute_offset(addr_base, addr_offset))
    }

    fn mem_read_u8(&self, addr_base: u16, addr_offset: i16) -> u8 {
        self.mem
            .read_u8(self.mem.compute_offset(addr_base, addr_offset))
    }

    fn mem_write_s16(&mut self, addr_base: u16, addr_offset: i16, value: s16) {
        self.mem
            .write_s16(self.mem.compute_offset(addr_base, addr_offset), value);
    }

    fn mem_write_u8(&mut self, addr_base: u16, addr_offset: i16, value: u8) {
        self.mem
            .write_u8(self.mem.compute_offset(addr_base, addr_offset), value);
    }

    #[allow(clippy::identity_op)]
    pub fn fetch(&mut self) {
        let lo = *self.mem.read_s16(self.pc + 2).as_u16() as u32;
        let hi = *self.mem.read_s16(self.pc + 0).as_u16() as u32;
        self.ir = (hi << 16) | (lo << 0);
    }

    fn handle_exn(&self, code: u16) {
        match code {
            exn_codes::ILLEGAL_INSTR => {
                eprintln!("Illegal instruction at pc={}: 0x{:X?}", self.pc, self.ir);
                std::process::exit(1);
            }

            exn_codes::DEBUG_BREAKPOINT => {
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

            exn_codes::DIV_BY_ZERO => {
                eprintln!("Division by zero at pc={}", self.pc);
                std::process::exit(1);
            }

            exn_codes::DEBUG_PUTS => {
                let s_ptr = self.regs.get(Reg::A0);
                let s_len = self.regs.get(Reg::A1);
                let s = (0..s_len)
                    .map(|i| self.mem_read_u8(s_ptr, i))
                    .map(char::from)
                    .collect::<String>();
                println!("DEBUG_PUTS: {}", s);
            }

            other => unimplemented!("unimplemented exception code `0x{:X?}`", other),
        }
    }
}

pub const ROM_SIZE: usize = 4 * KIB;
pub const USER_MEM_SIZE: usize = 54 * KIB;
pub const KERNEL_MEM_SIZE: usize = 4 * KIB;

pub trait MemRw {
    fn read_u8(&self, addr: u16) -> u8;
    fn write_u8(&mut self, addr: u16, value: u8);
    fn read_s16(&self, addr: u16) -> s16;
    fn write_s16(&mut self, addr: u16, value: s16);
}

pub struct Memory {
    pub mmio: Mmio,
    pub rom: MemBlock<ROM_SIZE>,
    pub user: MemBlock<USER_MEM_SIZE>,
    pub kernel: MemBlock<KERNEL_MEM_SIZE>,
}

impl Memory {
    pub const MMIO_START: u16 = 0;
    pub const MMIO_END: u16 = Self::MMIO_START + Mmio::SIZE - 1;
    pub const ROM_START: u16 = Mmio::SIZE;
    pub const ROM_END: u16 = Self::ROM_START + ROM_SIZE as u16 - 1;
    pub const USER_START: u16 = Self::ROM_START + ROM_SIZE as u16;
    pub const USER_END: u16 = Self::USER_START + USER_MEM_SIZE as u16 - 1;
    pub const KERNEL_START: u16 = Self::USER_START + USER_MEM_SIZE as u16;

    /// Creates a new memory instance with the given ROM.
    pub fn new(rom: MemBlock<ROM_SIZE>) -> Self {
        Self {
            mmio: Mmio::default(),
            rom,
            user: MemBlock::new_zeroed(),
            kernel: MemBlock::new_zeroed(),
        }
    }

    fn compute_offset(&self, addr_base: u16, addr_offset: i16) -> u16 {
        let addr_base = addr_base as i32;
        let addr_offset = addr_offset as i32;
        addr_base
            .checked_add(addr_offset)
            .expect("no overflow")
            .try_into()
            .expect("no overflow")
    }

    fn effective_addr(&self, addr: u16) -> (&dyn MemRw, u16) {
        match addr {
            Self::MMIO_START..=Self::MMIO_END => (&self.mmio, addr),
            Self::ROM_START..=Self::ROM_END => (&self.rom, addr - Self::ROM_START),
            Self::USER_START..=Self::USER_END => (&self.user, addr - Self::USER_START),
            Self::KERNEL_START.. => (&self.kernel, addr - Self::KERNEL_START),
        }
    }

    fn effective_addr_mut(&mut self, addr: u16) -> (&mut dyn MemRw, u16) {
        match addr {
            Self::MMIO_START..=Self::MMIO_END => (&mut self.mmio, addr),
            Self::ROM_START..=Self::ROM_END => (&mut self.rom, addr - Self::ROM_START),
            Self::USER_START..=Self::USER_END => (&mut self.user, addr - Self::USER_START),
            Self::KERNEL_START.. => (&mut self.kernel, addr - Self::KERNEL_START),
        }
    }
}

impl MemRw for Memory {
    fn read_u8(&self, addr: u16) -> u8 {
        let (seg, addr) = self.effective_addr(addr);
        seg.read_u8(addr)
    }

    fn write_u8(&mut self, addr: u16, value: u8) {
        let (seg, addr) = self.effective_addr_mut(addr);
        seg.write_u8(addr, value);
    }

    fn read_s16(&self, addr: u16) -> s16 {
        let (seg, addr) = self.effective_addr(addr);
        seg.read_s16(addr)
    }

    fn write_s16(&mut self, addr: u16, value: s16) {
        let (seg, addr) = self.effective_addr_mut(addr);
        seg.write_s16(addr, value);
    }
}

#[derive(Default)]
pub struct Mmio {}

impl Mmio {
    pub const SIZE: u16 = 2 * KIB as u16;
}

impl MemRw for Mmio {
    fn read_u8(&self, addr: u16) -> u8 {
        unimplemented!("unimplemented MMIO u8 read from address {}", addr);
    }

    fn write_u8(&mut self, addr: u16, value: u8) {
        use yansi::Paint;
        match addr {
            1 => println!(
                "MMIO[{addr}] <- {v}_u8 = {h} = {c:?}",
                addr = Paint::cyan(format!("0x{:04X}", addr)),
                v = Paint::green(value),
                h = Paint::green(format!("0x{:02X}", value)),
                c = value as char,
            ),
            _ => unimplemented!("unimplemented MMIO u8 write to address {}", addr),
        }
    }

    fn read_s16(&self, addr: u16) -> s16 {
        unimplemented!("unimplemented MMIO s16 read from address {}", addr);
    }

    fn write_s16(&mut self, addr: u16, value: s16) {
        use yansi::Paint;
        match addr {
            1 => println!(
                "MMIO[{addr}] <- {v}_u16 = {h} = {c:?}",
                addr = Paint::cyan(format!("0x{:04X}", addr)),
                v = Paint::green(value.as_u16()),
                h = Paint::green(format!("0x{:04X}", value.as_u16())),
                c = char::from_u32(*value.as_u16() as u32)
            ),
            _ => unimplemented!("unimplemented MMIO s16 write to address {}", addr),
        }
    }
}

pub struct MemBlock<const N: usize> {
    pub mem: Box<[u8; N]>,
}

impl<const N: usize> MemBlock<N> {
    pub fn new_zeroed() -> Self {
        Self {
            mem: Box::new([0; N]),
        }
    }

    pub fn from_vec(vec: Vec<u8>) -> Option<Self> {
        // Zeros out any remaining bytes beyond the end of the vec.
        let mut mem = Box::new([0; N]);
        if vec.len() > mem.len() {
            return None;
        }
        mem[..vec.len()].copy_from_slice(&vec);
        Some(Self { mem })
    }
}

impl<const N: usize> MemRw for MemBlock<N> {
    fn read_u8(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write_u8(&mut self, addr: u16, value: u8) {
        self.mem[addr as usize] = value;
    }

    #[allow(clippy::identity_op)]
    fn read_s16(&self, addr: u16) -> s16 {
        let lo = self.mem[addr as usize + 1] as u16;
        let hi = self.mem[addr as usize + 0] as u16;
        ((hi << 8) | (lo << 0)).into()
    }

    #[allow(clippy::identity_op)]
    fn write_s16(&mut self, addr: u16, value: s16) {
        let value: u16 = value.into();
        let hi = (value >> 8) as u8;
        let lo = (0x00FF & value) as u8;
        self.mem[addr as usize + 1] = lo;
        self.mem[addr as usize + 0] = hi;
    }
}
