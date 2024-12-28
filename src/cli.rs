//! Defines the `clap` command line interface for `lark-vm`.
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The path to the ROM file containing read-only code segment
    pub romfile: PathBuf,

    /// Start in debug mode?
    #[arg(short, long)]
    pub debug: bool,

    /// Before execution, print out a hexdump of ROM file.
    #[arg(short, long)]
    pub print_rom: bool,

    /// Path to the ROM source file (lark assembly or meadowlark).
    #[arg(short, long)]
    pub src_path: Option<PathBuf>,
}

impl Cli {
    pub fn rom_src_path(&self) -> PathBuf {
        self.src_path
            .as_ref()
            .map(Clone::clone)
            .unwrap_or_else(|| self.romfile.with_extension("").with_extension("lark"))
    }
}
