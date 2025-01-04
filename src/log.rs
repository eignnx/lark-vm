#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        print!("[LOG]: ");
        println!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_instr {
    ([$size:expr] $name:ident) => {
        $crate::cpu::LogMsg::Instr {
            size: $size,
            name: stringify!($name).to_string(),
            args: vec![]
        }
    };
    ([$size:expr] $name:ident $firstargval:expr $(, $argval:expr)*) => {
        $crate::cpu::LogMsg::Instr {
            size: $size,
            name: stringify!($name).to_string(),
            args: vec![
                match stringify!($firstargval) {
                    "rd" | "rs" | "rt" => {
                        (Some($crate::cpu::ArgStyle::Reg), $firstargval.to_string())
                    },
                    _ => {
                        (None, format!("{}={:?}", stringify!($firstargval), $firstargval))
                    }
                },

                $(
                    match stringify!($argval) {
                        "rd" | "rs" | "rt" => {
                            (Some($crate::cpu::ArgStyle::Reg), $argval.to_string())
                        },
                        _ => {
                            (None, format!("{}={}", stringify!($argval), $argval))
                        }
                    },
                )*
            ],
        }
    };
}
