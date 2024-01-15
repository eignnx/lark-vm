#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            print!("[LOG]: ");
            println!($($arg)*);
        }
    };
}

#[macro_export]
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
            use yansi::Paint;
            print!("[LOG]: ");
            print!("|{}|", $size);
            print!("\t{}\t", Paint::yellow(stringify!($name)).bold());
            match stringify!($firstargval) {
                "rd" | "rs" | "rt" => {
                    print!("{}", Paint::magenta(
                        <$crate::cpu::regs::Reg as std::convert::TryFrom<u8>>::try_from($firstargval as u8).unwrap().to_string()
                    ));
                },
                _ => {
                    print!("{}={}", stringify!($firstargval), $firstargval);
                }
            }
            $(
            match stringify!($argval) {
                "rd" | "rs" | "rt" => {
                    print!(", {}", Paint::magenta(
                        <$crate::cpu::regs::Reg as std::convert::TryFrom<u8>>::try_from($firstargval as u8).unwrap().to_string()
                    ));
                },
                _ => {
                    print!(", {}={}", stringify!($argval), $argval);
                }
            }
            )*
            println!();
        }
    };
}
