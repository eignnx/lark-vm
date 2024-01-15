use std::str::FromStr;

use crate::{cpu::regs, utils::s16};

use super::{regs::Reg, Cpu};

impl Cpu {
    /// Pauses execution until user presses enter.
    /// Allow the user to enter commands to query the state of the CPU.
    pub fn breakpoint(&mut self) {
        use std::io::{self, BufRead, Write};

        if !self.in_debug_mode {
            return;
        }

        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        let mut line = String::new();

        loop {
            print!("debug> ");

            io::stdout().flush().unwrap();
            stdin.read_line(&mut line).unwrap();

            if line.trim().is_empty() {
                break;
            }

            let cmd = DbgCmd::parse(&mut &line[..]).unwrap_or_else(|err| {
                eprintln!("error: {}", err);
                DbgCmd::Eval(DbgVal::U16(0))
            });

            self.eval_dbg_cmd(&cmd);

            if let DbgCmd::Continue = cmd {
                break;
            }

            line.clear();
        }
    }

    fn eval_dbg_cmd(&mut self, cmd: &DbgCmd) {
        match cmd {
            DbgCmd::Eval(DbgVal::Spr(Spr::Ir)) => println!("-> {:0b}", self.ir),
            DbgCmd::Eval(val) => println!("-> {}", self.eval_dbg_val_rvalue(val)),
            DbgCmd::Set(lhs, rhs) => {
                let rhs = self.eval_dbg_val_rvalue(rhs);
                let old = self.set_lvalue(lhs, rhs);
                println!("{old} -> {rhs}",)
            }
            DbgCmd::PrintStack { depth } => self.print_stack(*depth),
            DbgCmd::ListBreakpoints => {
                println!("breakpoints:");
                for (i, bp) in self.breakpoints.iter().enumerate() {
                    println!("\t #{}: 0x{:04X} = {}", i + 1, bp, bp);
                }
                if self.breakpoints.is_empty() {
                    println!("\t<no breakpoints set>");
                }
            }
            DbgCmd::AddBreakpoint(val) => {
                let address = self.eval_dbg_val_rvalue(val);
                self.breakpoints.insert(address);
                println!("added breakpoint at 0x{:04X} = {}", address, address);
            }
            DbgCmd::RemoveBreakpoint(val) => {
                let Some(index) = self.eval_dbg_val_rvalue(val).checked_sub(1) else {
                    println!("Invalid breakpoint ordinal. Enter a value between 1 and {}.", self.breakpoints.len());
                    return;
                };
                let address = *self.breakpoints.iter().nth(index as usize).unwrap();
                self.breakpoints.remove(&address);
                println!(
                    "removed breakpoint #{}: 0x{:04X} = {}",
                    index + 1,
                    address,
                    address
                );
            }
            DbgCmd::Continue => {
                self.in_debug_mode = false;
                println!("continuing execution...");
            }
            DbgCmd::PrintRegs => {
                println!("general-purpose registers:");
                for (regname, regval) in self.regs.iter() {
                    println!("\t${regname} = 0x{v:04X} = {v}", v = regval.as_u16());
                }
                println!("special-purpose registers:");
                println!("\t${} = 0x{v:04X} = {v}", Spr::Lo, v = *self.lo.as_u16());
                println!("\t${} = 0x{v:04X} = {v}", Spr::Hi, v = *self.hi.as_u16());
                println!("\t${} = 0x{v:04X} = {v}", Spr::Pc, v = self.pc,);
                println!("\t${} = 0x{v:08X} = {v} = 0b{v:032b}", Spr::Ir, v = self.ir);
            }
        }
    }

    fn eval_dbg_val_rvalue(&mut self, val: &DbgVal) -> u16 {
        match val {
            DbgVal::U16(val) => *val,
            DbgVal::Gpr(reg) => self.regs.get(*reg),
            DbgVal::Spr(spr) => match spr {
                Spr::Pc => self.pc,
                Spr::Ir => unreachable!(),
                Spr::Lo => *self.lo.as_u16(),
                Spr::Hi => *self.hi.as_u16(),
            },
            DbgVal::Mem { base, offset } => {
                let base = self.eval_dbg_val_rvalue(base);
                let offset = self.eval_dbg_val_rvalue(offset) as i16;
                *self.mem_read_s16(base, offset).as_u16()
            }
            DbgVal::Neg(val) => self.eval_dbg_val_rvalue(val).wrapping_neg(),
        }
    }

    /// Returns the the previous value of the lvalue.
    fn set_lvalue(&mut self, lhs: &DbgVal, rhs: u16) -> u16 {
        match lhs {
            DbgVal::Gpr(reg) => {
                let prev = self.regs.get(*reg);
                self.regs.set(*reg, rhs);
                prev
            }
            DbgVal::Spr(Spr::Ir) => panic!("cannot assign to $ir"),
            DbgVal::Spr(spr) => {
                let prev = match spr {
                    Spr::Pc => self.pc,
                    Spr::Ir => unreachable!(),
                    Spr::Lo => *self.lo.as_u16(),
                    Spr::Hi => *self.hi.as_u16(),
                };
                match spr {
                    Spr::Pc => self.pc = rhs,
                    Spr::Ir => unreachable!(),
                    Spr::Lo => *self.lo.as_u16_mut() = rhs,
                    Spr::Hi => *self.hi.as_u16_mut() = rhs,
                }
                prev
            }
            DbgVal::Mem { base, offset } => {
                let base = self.eval_dbg_val_rvalue(base);
                let offset = *s16::from(self.eval_dbg_val_rvalue(offset)).as_i16();
                let prev = *self.mem_read_s16(base, offset).as_u16();
                self.mem_write_s16(base, offset, rhs.into());
                prev
            }
            DbgVal::U16(_) | DbgVal::Neg(_) => panic!("cannot assign to rvalue"),
        }
    }

    fn print_stack(&self, depth: u16) {
        let sp = self.regs.get(Reg::Sp);
        let mut addr = sp;
        for i in 0..depth {
            let value = *self.mem_read_s16(addr, 0).as_u16();
            println!("[$sp+{:02}] = 0x{value:04X} = {value:06}", 2 * i);
            addr += 2;
        }
    }
}

#[derive(Debug, Clone)]
enum DbgCmd {
    Eval(DbgVal),
    Set(DbgVal, DbgVal),
    PrintStack { depth: u16 },
    ListBreakpoints,
    AddBreakpoint(DbgVal),
    RemoveBreakpoint(DbgVal),
    Continue,
    PrintRegs,
}

impl DbgCmd {
    fn parse(s: &mut &str) -> winnow::PResult<Self> {
        use winnow::ascii::{dec_uint, multispace0, multispace1};
        use winnow::combinator::{alt, opt, preceded, separated_pair};
        use winnow::Parser;

        alt((
            // Try parsing a set command.
            separated_pair(
                DbgVal::parse,
                (multispace0, "=", multispace0),
                DbgVal::parse,
            )
            .map(|(lhs, rhs)| Self::Set(lhs, rhs)),
            // Try parsing an eval command.
            DbgVal::parse.map(Self::Eval),
            // Try parsing a print stack command.
            preceded(("stack", multispace0), opt(dec_uint)).map(|val: Option<u16>| {
                Self::PrintStack {
                    depth: val.unwrap_or(4),
                }
            }),
            // Try parsing an add breakpoint command.
            preceded(
                (alt(("+b", "b", "breakpoint", "+breakpoint")), multispace1),
                DbgVal::parse.map(Self::AddBreakpoint),
            ),
            // Try parsing a remove breakpoint command.
            preceded(
                (alt(("-b", "-breakpoint")), multispace1, opt("#")),
                DbgVal::parse.map(Self::RemoveBreakpoint),
            ),
            // Try parsing a list breakpoints command.
            alt(("b", "breakpoints")).map(|_| Self::ListBreakpoints),
            alt(("c", "continue")).map(|_| Self::Continue),
            alt(("r", "regs")).map(|_| Self::PrintRegs),
        ))
        .parse_next(s)
    }
}

#[derive(Debug, Clone)]
enum DbgVal {
    /// The value held in a general-purpose register.
    Gpr(Reg),
    /// The value held in a special-purpose register.
    Spr(Spr),
    /// The value in memory at the given base address with the given offset.
    Mem {
        base: Box<DbgVal>,
        offset: Box<DbgVal>,
    },
    /// An integer value.
    U16(u16),
    /// The negation of a value.
    Neg(Box<DbgVal>),
}

impl DbgVal {
    fn parse(s: &mut &str) -> winnow::PResult<Self> {
        use winnow::{
            ascii::{dec_uint, hex_uint},
            combinator::{alt, delimited, opt, preceded},
            Parser,
        };

        alt((
            // If the string starts with `[` parse as `[addr]`.
            delimited(
                '[',
                (
                    Self::parse,
                    opt(alt((
                        preceded('+', Self::parse),
                        preceded('-', Self::parse).map(|val| Self::Neg(Box::new(val))),
                    ))),
                )
                    .map(|(base, offset)| (base, offset.unwrap_or(Self::U16(0)))),
                ']',
            )
            .map(|(base, offset)| Self::Mem {
                base: Box::new(base),
                offset: Box::new(offset),
            }),
            // Try parsing an integer.
            alt((preceded("0x", hex_uint), dec_uint)).map(Self::U16),
            // Try parsing a register.
            preceded(
                opt('$'),
                alt((
                    // Try parsing a general-purpose register.
                    alt(regs::REG_NAMES).parse_to().map(Self::Gpr),
                    // Try parsing a special-purpose register.
                    alt(SPR_NAMES).parse_to().map(Self::Spr),
                )),
            ),
        ))
        .parse_next(s)
    }
}

#[derive(Debug, Clone)]
enum Spr {
    Pc,
    Ir,
    Lo,
    Hi,
}

impl std::fmt::Display for Spr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Pc => "pc",
            Self::Ir => "ir",
            Self::Lo => "lo",
            Self::Hi => "hi",
        };
        write!(f, "{}", name)
    }
}

const SPR_NAMES: [&str; 4] = ["pc", "ir", "lo", "hi"];

impl FromStr for Spr {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let stripped = s.strip_prefix('$').unwrap_or(s);
        match stripped {
            "pc" => Ok(Self::Pc),
            "ir" => Ok(Self::Ir),
            "lo" => Ok(Self::Lo),
            "hi" => Ok(Self::Hi),
            _ => Err(format!("invalid special-purpose register name: `{}`", s)),
        }
    }
}
