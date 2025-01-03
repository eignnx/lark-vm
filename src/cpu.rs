use std::{
    cell::RefCell,
    collections::BTreeSet,
    path::PathBuf,
    rc::Rc,
    sync::mpsc::{Receiver, Sender},
};

use self::{dex::DexErr, interrupts::Interrupt, regs::RegisterFile};
use crate::utils::s16;

mod debugger;
mod decode;
mod dex;
mod exn_codes;
pub mod instr;
pub mod interrupts;
mod opcodes;
mod regs;

pub const KIB: usize = 1024;
pub const STACK_INIT: u16 = Memory::USER_END - 1;

pub const VTTY_COLS: usize = 80;
pub const VTTY_ROWS: usize = 24;
pub const VTTY_BYTES: usize = VTTY_COLS * VTTY_ROWS;

pub enum ArgStyle {
    Imm,
    Reg,
}

pub enum LogMsg {
    /// Once an instruction is decoded, it can be logged with this.
    Instr {
        size: u16,
        name: String,
        args: Vec<(Option<ArgStyle>, String)>,
    },

    /// Signals that a MMIO read has occurred.
    MmioRead {
        addr: u16,
        value: String,
    },

    /// Signals that a MMIO write has occurred.
    MmioWrite {
        addr: u16,
        value: String,
    },

    /// For printing a string to the command log.
    DebugPuts {
        addr: u16,
        value: String,
    },
    Error(String),
}

pub enum Signal {
    /// Signals that the CPU should halt.
    Halt,
    /// Requests a log message be printed.
    Log(LogMsg),
    /// Pauses execution of the VM allowing the user to interact with the
    /// debugger.
    Breakpoint,
    /// Signals that an illegal instruction has been executed.
    IllegalInstr,
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

    pub supervisor: Sender<Signal>,
    pub pending_interrupts: Receiver<Interrupt>,
    pub interrupt_return_address: u16,
    pub interrupts_enabled: bool,

    pub in_debug_mode: bool,
    pub breakpoints: BTreeSet<u16>,
    pub rom_src_path: Option<PathBuf>,
}

impl Cpu {
    pub fn new(
        rom: MemBlock<ROM_SIZE>,
        vtty_buf: Rc<RefCell<MemBlock<VTTY_BYTES>>>,
        logger: Sender<Signal>,
        interrupt_channel: Receiver<Interrupt>,
    ) -> Self {
        Self {
            regs: RegisterFile::new(STACK_INIT),
            pc: Memory::ROM_START,
            ir: 0,
            hi: s16::default(),
            lo: s16::default(),
            mem: Memory::new(rom, vtty_buf),

            supervisor: logger,
            pending_interrupts: interrupt_channel,
            interrupt_return_address: 0x0000,
            interrupts_enabled: true,

            in_debug_mode: false,
            breakpoints: BTreeSet::new(),
            rom_src_path: None,
        }
    }

    pub fn reset(&mut self) {
        self.regs.reset(STACK_INIT);
        self.pc = Memory::ROM_START;
        self.ir = 0;
        self.hi = s16::default();
        self.lo = s16::default();
        self.in_debug_mode = false;
        self.interrupt_return_address = 0x0000;
        self.interrupts_enabled = true;
        self.mem.reset();
    }

    pub fn load_rom(&mut self, rom: MemBlock<ROM_SIZE>) {
        self.mem.rom = rom;
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

    pub fn step(&mut self) -> Result<(), DexErr> {
        // First check for interrupts.
        if self.interrupts_enabled {
            // If there are interrupts pending, send ONE (1) to the CPU.
            if let Ok(interrupt) = self.pending_interrupts.try_recv() {
                self.send_interrupt(interrupt);
            }
        }

        self.fetch();

        if self.breakpoints.contains(&self.pc) {
            self.in_debug_mode = true;
        }

        self.decode_and_execute()?;
        Ok(())
    }

    pub fn run(&mut self) {
        loop {
            if let Err(e) = self.step() {
                self.log(LogMsg::Error(format!("{:?}", e)));
            }
        }
    }

    pub fn signal(&self, sig: Signal) {
        self.supervisor.send(sig).unwrap();
    }

    pub fn log(&self, msg: LogMsg) {
        self.signal(Signal::Log(msg));
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
        let lo = self.mem.read_s16(self.pc + 2).as_u16() as u32;
        let hi = self.mem.read_s16(self.pc + 0).as_u16() as u32;
        self.ir = (hi << 16) | (lo << 0);
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
    pub fn new(rom: MemBlock<ROM_SIZE>, vtty_buf: Rc<RefCell<MemBlock<VTTY_BYTES>>>) -> Self {
        Self {
            mmio: Mmio::new(vtty_buf),
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

    fn reset(&mut self) {
        self.rom = MemBlock::new_zeroed();
        self.user = MemBlock::new_zeroed();
        self.kernel = MemBlock::new_zeroed();
        // Leave MMIO alone.
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

pub const VTTY_START: u16 = 128;
pub const VTTY_END: u16 = VTTY_START + VTTY_BYTES as u16 - 1;

pub struct Mmio {
    vtty_buf: Rc<RefCell<MemBlock<VTTY_BYTES>>>,
}

impl Mmio {
    pub const SIZE: u16 = 2 * KIB as u16;

    pub fn new(vtty_buf: Rc<RefCell<MemBlock<VTTY_BYTES>>>) -> Self {
        Self { vtty_buf }
    }
}

impl MemRw for Mmio {
    fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            VTTY_START..=VTTY_END => {
                let addr = addr - VTTY_START;
                let vtty_buf = self.vtty_buf.borrow();
                vtty_buf.read_u8(addr)
            }
            _ => unimplemented!("unimplemented MMIO u8 read from address {}", addr),
        }
    }

    fn write_u8(&mut self, addr: u16, value: u8) {
        match addr {
            1 => {} // TODO
            VTTY_START..=VTTY_END => {
                let addr = addr - VTTY_START;
                let mut vtty_buf = self.vtty_buf.borrow_mut();
                vtty_buf.write_u8(addr, value);
            }
            _ => unimplemented!("unimplemented MMIO u8 write to address {}", addr),
        }
    }

    fn read_s16(&self, addr: u16) -> s16 {
        unimplemented!("unimplemented MMIO s16 read from address {}", addr);
    }

    #[allow(clippy::identity_op)]
    fn write_s16(&mut self, addr: u16, value: s16) {
        match addr {
            1 => {} // TODO
            VTTY_START..=VTTY_END => {
                let addr = addr - VTTY_START;
                let value = value.as_u16();
                let value_lo = (value & 0x00FF) as u8;
                let value_hi = (value >> 8) as u8;
                let mut vtty_buf = self.vtty_buf.borrow_mut();
                vtty_buf.write_u8(addr + 0, value_lo);
                vtty_buf.write_u8(addr + 1, value_hi);
            }
            _ => unimplemented!("unimplemented MMIO s16 write to address {}", addr),
        }
    }
}

pub struct MemBlock<const N: usize> {
    pub mem: Box<[u8; N]>,
}

impl<const N: usize> Default for MemBlock<N> {
    fn default() -> Self {
        Self {
            mem: Box::new([0; N]),
        }
    }
}

impl<const N: usize> MemBlock<N> {
    pub fn new_zeroed() -> Self {
        Self::default()
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
    #[track_caller]
    fn read_u8(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    #[track_caller]
    fn write_u8(&mut self, addr: u16, value: u8) {
        self.mem[addr as usize] = value;
    }

    #[track_caller]
    #[allow(clippy::identity_op)]
    fn read_s16(&self, addr: u16) -> s16 {
        let lo = self.mem[addr as usize + 1] as u16;
        let hi = self.mem[addr as usize + 0] as u16;
        ((hi << 8) | (lo << 0)).into()
    }

    #[track_caller]
    #[allow(clippy::identity_op)]
    fn write_s16(&mut self, addr: u16, value: s16) {
        let value: u16 = value.into();
        let hi = (value >> 8) as u8;
        let lo = (0x00FF & value) as u8;
        self.mem[addr as usize + 1] = lo;
        self.mem[addr as usize + 0] = hi;
    }
}
