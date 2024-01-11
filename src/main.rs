#![allow(dead_code)]

use std::{mem, path::PathBuf};

use bitvec::prelude::*;

mod decode;
mod exn_codes;
mod opcodes;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            print!("[LOG]: ");
            println!($($arg)*);
        }
    };
}

macro_rules! log_instr {
    ([$size:expr] $name:ident) => {
        if cfg!(debug_assertions) {
            print!("[LOG]: ");
            print!("|{}|", $size);
            print!("\t{}", stringify!($name));
            println!();
        }
    };
    ([$size:expr] $name:ident $firstargval:expr $(, $argval:expr)*) => {
        if cfg!(debug_assertions) {
            print!("[LOG]: ");
            print!("|{}|", $size);
            print!("\t{}\t", stringify!($name));
            print!("{}={}", stringify!($firstargval), $firstargval);
            $(
                print!(", {}={}", stringify!($argval), $argval);
            )*
            println!();
        }
    };
}

fn main() {
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

const KIB: usize = 1024;

/// Represents a signed or unsigned 16-bit number. The sign is determined by the
/// user.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub union s16 {
    u16: u16,
    i16: i16,
}

impl s16 {
    pub const ZERO: Self = Self { u16: 0 };
}

impl Default for s16 {
    fn default() -> Self {
        Self::ZERO
    }
}

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

    pub rom_src_path: Option<PathBuf>,
}

impl Cpu {
    pub fn new(start_addr: u16, rom: MemBlock<ROM_SIZE>, rom_src_path: Option<PathBuf>) -> Self {
        Self {
            regs: RegisterFile {
                indexed_u16: [0; 15],
            },
            pc: start_addr,
            ir: 0,
            hi: s16::default(),
            lo: s16::default(),
            mem: Memory::new(rom),
            rom_src_path,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.fetch();
            self.decode_and_execute();
        }
    }

    fn mem_read_u16(&self, addr_base: u16, addr_offset: i16) -> u16 {
        self.mem
            .read_u16(self.mem.compute_offset(addr_base, addr_offset))
    }

    fn mem_read_u8(&self, addr_base: u16, addr_offset: i16) -> u8 {
        self.mem
            .read_u8(self.mem.compute_offset(addr_base, addr_offset))
    }

    fn mem_write_u16(&mut self, addr_base: u16, addr_offset: i16, value: u16) {
        self.mem
            .write_u16(self.mem.compute_offset(addr_base, addr_offset), value);
    }

    fn mem_write_u8(&mut self, addr_base: u16, addr_offset: i16, value: u8) {
        self.mem
            .write_u8(self.mem.compute_offset(addr_base, addr_offset), value);
    }

    #[allow(clippy::identity_op)]
    pub fn fetch(&mut self) {
        let lo = self.mem.read_u16(self.pc + 2) as u32;
        let hi = self.mem.read_u16(self.pc + 0) as u32;
        self.ir = (hi << 16) | (lo << 0);
    }

    pub fn decode_and_execute(&mut self) {
        let ir = self.ir.view_bits::<Msb0>();

        // Opcode is most significant 6 bits.
        let opcode = ir[..6].load::<u8>();
        let ir = &ir[6..];

        if cfg!(debug_assertions) {
            print!("pc={}\t", self.pc);
        }

        match opcode {
            opcodes::EXN => {
                let (size, imm10) = decode::imm10(ir);
                log_instr!([size] exn imm10);
                self.handle_exn(imm10);
                self.pc += size;
            }

            opcodes::HALT => {
                log_instr!([1] halt);
                std::process::exit(0);
            }

            opcodes::LI => {
                let (size, rd, simm16) = decode::reg_simm(ir);
                log_instr!([size] li rd, simm16);
                self.regs.write_i16(rd, simm16);
                self.pc += size;
            }

            opcodes::ADD => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] add rd, rs, rt);
                let sum = self.regs.read_i16(rs) + self.regs.read_i16(rt);
                self.regs.write_i16(rd, sum);
                self.pc += size;
            }

            opcodes::ADDU => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] addu rd, rs, rt);
                let sum = self.regs.read_u16(rs) + self.regs.read_u16(rt);
                self.regs.write_u16(rd, sum);
                self.pc += size;
            }

            opcodes::ADDI => {
                let (size, rd, rs, imm) = decode::reg_reg_simm(ir);
                log_instr!([size] addi rd, rs, imm);
                let sum = self.regs.read_i16(rs) + imm;
                self.regs.write_i16(rd, sum);
                self.pc += size;
            }

            opcodes::SUB => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] sub rd, rs, rt);
                let sum = self.regs.read_i16(rs) - self.regs.read_i16(rt);
                self.regs.write_i16(rd, sum);
                self.pc += size;
            }

            opcodes::SUBU => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] subu rd, rs, rt);
                let sum = self.regs.read_u16(rs) - self.regs.read_u16(rt);
                self.regs.write_u16(rd, sum);
                self.pc += size;
            }

            opcodes::SUBI => {
                let (size, rd, rs, imm) = decode::reg_reg_simm(ir);
                log_instr!([size] subi rd, rs, imm);
                let sum = self.regs.read_i16(rs) - imm;
                self.regs.write_i16(rd, sum);
                self.pc += size;
            }

            opcodes::JR => {
                let (size, rs) = decode::reg(ir);
                log_instr!([size] jr rs);
                self.pc = self.regs.read_u16(rs);
            }

            opcodes::J => {
                let (size, offset) = decode::simm16(ir);
                log_instr!([size] j offset);
                self.pc = (self.pc as i32)
                    .checked_add(offset as i32)
                    .expect("Jump address overflow") as u16;
            }

            // Jump and link.
            // Example: jal $rd, ADDR
            opcodes::JAL => {
                let (size, rd, offset) = decode::reg_simm(ir);
                log_instr!([size] jal rd, offset);
                self.regs.write_u16(rd, self.pc + size);
                self.pc = (self.pc as i32 + offset as i32) as u16;
            }

            // Jump register and link.
            // Example: jral $rd, $rs
            //                |    |
            //          save pc    jump address
            opcodes::JRAL => {
                let (size, rd, rs) = decode::reg_reg(ir);
                log_instr!([size] jral rd, rs);
                self.regs.write_u16(rd, self.pc + size);
                self.pc = self.regs.read_u16(rs);
            }

            opcodes::TLT => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] tlt rd, rs, rt);
                let value = self.regs.read_i16(rs) < self.regs.read_i16(rt);
                self.regs.write_u16(rd, value as u16);
                self.pc += size;
            }

            opcodes::TLTU => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] tltu rd, rs, rt);
                let value = self.regs.read_u16(rs) < self.regs.read_u16(rt);
                self.regs.write_u16(rd, value as u16);
                self.pc += size;
            }

            opcodes::TGE => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] tge rd, rs, rt);
                let value = self.regs.read_i16(rs) >= self.regs.read_i16(rt);
                self.regs.write_u16(rd, value as u16);
                self.pc += size;
            }

            opcodes::TGEU => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] tgeu rd, rs, rt);
                let value = self.regs.read_u16(rs) >= self.regs.read_u16(rt);
                self.regs.write_u16(rd, value as u16);
                self.pc += size;
            }

            opcodes::TEQ => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] teq rd, rs, rt);
                let value = self.regs.read_i16(rs) == self.regs.read_i16(rt);
                self.regs.write_u16(rd, value as u16);
                self.pc += size;
            }

            opcodes::TNE => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] tne rd, rs, rt);
                let value = self.regs.read_i16(rs) != self.regs.read_i16(rt);
                self.regs.write_u16(rd, value as u16);
                self.pc += size;
            }

            opcodes::TEZ => {
                let (size, rd, rs) = decode::reg_reg(ir);
                log_instr!([size] tez rd, rs);
                let value = self.regs.read_i16(rs) == 0;
                self.regs.write_u16(rd, value as u16);
                self.pc += size;
            }

            opcodes::TNZ => {
                let (size, rd, rs) = decode::reg_reg(ir);
                log_instr!([size] tnz rd, rs);
                let value = self.regs.read_i16(rs) != 0;
                self.regs.write_u16(rd, value as u16);
                self.pc += size;
            }

            // Branch if false.
            opcodes::BF => {
                let (size, rs, addr_offset) = decode::reg_simm(ir);
                log_instr!([size] bf rs, addr_offset);
                if self.regs.read_u16(rs) == 0 {
                    self.pc = (self.pc as i32 + addr_offset as i32) as u16;
                } else {
                    self.pc += size;
                }
            }

            // Branch if true.
            opcodes::BT => {
                let (size, rs, addr_offset) = decode::reg_simm(ir);
                log_instr!([size] bt rs, addr_offset);
                if self.regs.read_u16(rs) != 0 {
                    self.pc = (self.pc as i32 + addr_offset as i32) as u16;
                } else {
                    self.pc += size;
                }
            }

            opcodes::NOT => {
                let (size, rd, rs) = decode::reg_reg(ir);
                log_instr!([size] not rd, rs);
                let value = self.regs.read_u16(rs) == 0;
                self.regs.write_u16(rd, value as u16);
                self.pc += size;
            }

            opcodes::NOP => {
                log_instr!([1] nop);
                self.pc += 1;
            }

            opcodes::MUL => {
                let (size, rs, rt) = decode::reg_reg(ir);
                log_instr!([size] mul rs, rt);

                let product = self.regs.read_i16(rs) as i32 * self.regs.read_i16(rt) as i32;
                let product = unsafe { mem::transmute::<i32, u32>(product) };
                let product: &BitSlice<u32, Lsb0> = product.view_bits();

                self.lo.i16 = product[0..16].load();
                self.hi.i16 = product[16..32].load();

                self.pc += size;
            }

            opcodes::MULU => {
                let (size, rs, rt) = decode::reg_reg(ir);
                log_instr!([size] mulu rs, rt);

                let product: u32 = self.regs.read_u16(rs) as u32 * self.regs.read_u16(rt) as u32;
                let product: &BitSlice<u32, Lsb0> = product.view_bits();

                self.lo.u16 = product[0..16].load();
                self.hi.u16 = product[16..32].load();

                self.pc += size;
            }

            opcodes::MVLO => {
                let (size, rd) = decode::reg(ir);
                log_instr!([size] mvlo rd);
                self.regs.write_u16(rd, unsafe { self.lo.u16 });
                self.pc += size;
            }

            opcodes::MVHI => {
                let (size, rd) = decode::reg(ir);
                log_instr!([size] mvhi rd);
                self.regs.write_u16(rd, unsafe { self.hi.u16 });
                self.pc += size;
            }

            opcodes::LW => {
                let (size, rd, rs, addr_offset) = decode::reg_reg_simm(ir);
                log_instr!([size] lw rd, rs, addr_offset);
                let addr_base = self.regs.read_u16(rs);
                let value = self.mem_read_u16(addr_base, addr_offset);
                self.regs.write_u16(rd, value);
                self.pc += size;
            }

            opcodes::LBU => {
                let (size, rd, rs, addr_offset) = decode::reg_reg_simm(ir);
                log_instr!([size] lbu rd, rs, addr_offset);
                let addr_base = self.regs.read_u16(rs);
                let value = self.mem_read_u8(addr_base, addr_offset);
                self.regs.write_u16(rd, value as u16);
                self.pc += size;
            }

            // Stores a word in memory given a address register and an offset.
            // Example: sw -32($t0), $t1
            //              ^   ^     ^
            //              |   |     |
            //         simm10   |     |
            //    base addr reg (rd)  |
            //              value reg (rs)
            opcodes::SW => {
                let (size, rd, rs, addr_offset) = decode::reg_reg_simm(ir);
                log_instr!([size] sw rd, rs, addr_offset);
                let addr_base = self.regs.read_u16(rd);
                let value = self.regs.read_u16(rs);
                self.mem_write_u16(addr_base, addr_offset, value);
                self.pc += size;
            }

            opcodes::SB => {
                let (size, rd, rs, addr_offset) = decode::reg_reg_simm(ir);
                log_instr!([size] sb rd, rs, addr_offset);
                let addr_base = self.regs.read_u16(rd);
                let value = (self.regs.read_u16(rs) & 0x00FF) as u8;
                self.mem_write_u8(addr_base, addr_offset, value);
                self.pc += size;
            }

            opcodes::SEB => {
                let (size, rd, rs) = decode::reg_reg(ir);
                log_instr!([size] seb rd, rs);
                let value = (self.regs.read_u16(rs) & 0x00FF) as u8;
                let value = unsafe { std::mem::transmute::<u8, i8>(value) };
                let value = value as i16;
                self.regs.write_i16(rd, value);
                self.pc += size;
            }

            opcodes::SHL => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] shl rd, rs, rt);
                let value = self.regs.read_u16(rs) << self.regs.read_u16(rt);
                self.regs.write_u16(rd, value);
                self.pc += size;
            }

            opcodes::SHR => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] shr rd, rs, rt);
                let value = self.regs.read_u16(rs) >> self.regs.read_u16(rt);
                self.regs.write_u16(rd, value);
                self.pc += size;
            }

            // Shift right arithmetic.
            opcodes::SHRA => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir);
                log_instr!([size] shra rd, rs, rt);
                // Will perform sign-extension after shifting.
                let value = self.regs.read_i16(rs) >> self.regs.read_u16(rt);
                self.regs.write_i16(rd, value);
                self.pc += size;
            }

            other => unimplemented!("unimplemented opcode `0x{:X?}` (pc={})", other, self.pc),
        }
    }

    fn handle_exn(&self, code: u16) {
        match code {
            exn_codes::ILLEGAL_INSTR => {
                eprintln!("Illegal instruction at pc={}: 0x{:X?}", self.pc, self.ir);
                std::process::exit(1);
            }
            exn_codes::DEBUG_BREAKPOINT => {
                let lineno = unsafe { self.regs.named.a0 };
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
            other => unimplemented!("unimplemented exception code `0x{:X?}`", other),
        }
    }
}

pub union RegisterFile {
    named: Regs,
    indexed_u16: [u16; 15],
    indexed_i16: [i16; 15],
}

impl RegisterFile {
    /// Writes a value to a register. The `rd` parameter is the destination
    /// register number. The `value` parameter is the value to be written.
    pub fn write_u16(&mut self, rd: u8, value: u16) {
        // Skip the zero register.
        if let Some(rd) = rd.checked_sub(1) {
            unsafe {
                *self
                    .indexed_u16
                    .get_mut(rd as usize)
                    .unwrap_or_else(|| panic!("Invalid register id: {rd}")) = value;
            }
        }
    }

    pub fn write_i16(&mut self, rd: u8, value: i16) {
        // Skip the zero register.
        if let Some(rd) = rd.checked_sub(1) {
            unsafe {
                *self
                    .indexed_i16
                    .get_mut(rd as usize)
                    .unwrap_or_else(|| panic!("Invalid register id: {rd}")) = value;
            }
        }
    }

    /// Reads a value from a register. The `rs` parameter is the source
    /// register number.
    pub fn read_i16(&self, rs: u8) -> i16 {
        if let Some(rs) = rs.checked_sub(1) {
            unsafe {
                *self
                    .indexed_i16
                    .get(rs as usize)
                    .unwrap_or_else(|| panic!("Invalid register id: {rs}"))
            }
        } else {
            // Register 0 is always zero.
            0
        }
    }

    pub fn read_u16(&self, rs: u8) -> u16 {
        if let Some(rs) = rs.checked_sub(1) {
            unsafe {
                *self
                    .indexed_u16
                    .get(rs as usize)
                    .unwrap_or_else(|| panic!("Invalid register id: {rs}"))
            }
        } else {
            // Register 0 is always zero.
            0
        }
    }
}

#[derive(Clone, Copy)]
pub struct Regs {
    /// Return value
    pub rv: i16,
    /// Return address
    pub ra: u16,

    /// Argument registers
    pub a0: i16,
    pub a1: i16,
    pub a2: i16,

    /// Callee-saved registers
    pub s0: i16,
    pub s1: i16,
    pub s2: i16,

    /// Temporaries
    pub t0: i16,
    pub t1: i16,
    pub t2: i16,

    /// Kernel reserved registers
    pub k0: i16,
    pub k1: i16,

    /// Process memory base pointer
    pub gp: u16,

    /// Stack pointer
    pub sp: u16,
}

const ROM_SIZE: usize = 4 * KIB;
const USER_MEM_SIZE: usize = 54 * KIB;
const KERNEL_MEM_SIZE: usize = 4 * KIB;

trait MemRw {
    fn read_u8(&self, addr: u16) -> u8;
    fn write_u8(&mut self, addr: u16, value: u8);
    fn read_u16(&self, addr: u16) -> u16;
    fn write_u16(&mut self, addr: u16, value: u16);
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

    fn read_u16(&self, addr: u16) -> u16 {
        let (seg, addr) = self.effective_addr(addr);
        seg.read_u16(addr)
    }

    fn write_u16(&mut self, addr: u16, value: u16) {
        let (seg, addr) = self.effective_addr_mut(addr);
        seg.write_u16(addr, value);
    }
}

#[derive(Default)]
pub struct Mmio {}

impl Mmio {
    pub const SIZE: u16 = 2 * KIB as u16;
}

impl MemRw for Mmio {
    fn read_u8(&self, _addr: u16) -> u8 {
        todo!()
    }

    fn write_u8(&mut self, addr: u16, value: u8) {
        match addr {
            1 => println!("MMIO[{addr}] = 0x{value:02X}, {:?}", value as char),
            _ => unimplemented!("unimplemented MMIO write to address {}", addr),
        }
    }

    fn read_u16(&self, _addr: u16) -> u16 {
        todo!()
    }

    fn write_u16(&mut self, _addr: u16, _value: u16) {
        todo!()
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
    fn read_u16(&self, addr: u16) -> u16 {
        let lo = self.mem[addr as usize + 1] as u16;
        let hi = self.mem[addr as usize + 0] as u16;
        (hi << 8) | (lo << 0)
    }

    #[allow(clippy::identity_op)]
    fn write_u16(&mut self, addr: u16, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (0x00FF & value) as u8;
        self.mem[addr as usize + 0] = lo;
        self.mem[addr as usize + 1] = hi;
    }
}
