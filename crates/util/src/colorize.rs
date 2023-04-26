use std::fmt;
use std::fmt::Formatter;
use crate::{Conclusion, PartialConclusion};

impl fmt::Display for Conclusion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let colored_str = match self {
            Conclusion::SUCCESS => "SUCCESS".to_green(),
            Conclusion::FAILED => "FAILED".to_red(),
        };

        f.write_str(&colored_str)
    }
}

impl fmt::Display for PartialConclusion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let colored_str = match self {
            PartialConclusion::INIT => format!(" {}", "∅".to_green()),
            PartialConclusion::SUCCESS => format!(" {}", "✓".to_green()),
            PartialConclusion::FAILED => format!(" {}", "✕".to_red()),
            PartialConclusion::CACHED => format!(" {}", "❤".to_blue()),
        };

        f.write_str(&colored_str)
    }
}

trait Colorize {
    fn to_black(&self) -> String;
    fn to_red(&self) -> String;
    fn to_green(&self) -> String;
    fn to_yellow(&self) -> String;
    fn to_blue(&self) -> String;
    fn to_purple(&self) -> String;
    fn to_turquoise(&self) -> String;
    fn to_gray(&self) -> String;
    fn to_white(&self) -> String;
}

impl Colorize for &str {
    fn to_black(&self) -> String { format!("\x1b[30m{self}\x1b[0m") }
    fn to_red(&self) -> String { format!("\x1b[31m{self}\x1b[0m") }
    fn to_green(&self) -> String { format!("\x1b[32m{self}\x1b[0m") }
    fn to_yellow(&self) -> String { format!("\x1b[33m{self}\x1b[0m") }
    fn to_blue(&self) -> String { format!("\x1b[34m{self}\x1b[0m") }
    fn to_purple(&self) -> String { format!("\x1b[35m{self}\x1b[0m") }
    fn to_turquoise(&self) -> String { format!("\x1b[36m{self}\x1b[0m") }
    fn to_gray(&self) -> String { format!("\x1b[37m{self}\x1b[0m") }
    fn to_white(&self) -> String { format!("\x1b[38m{self}\x1b[0m") }
}


