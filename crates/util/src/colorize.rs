use std::fmt;
use std::fmt::{Display, Formatter};

use crate::{Conclusion, PartialConclusion};

impl Display for Conclusion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let colored_str = match self {
            Conclusion::SUCCESS => "SUCCESS".as_green(),
            Conclusion::FAILED => "FAILED".as_red(),
        };

        f.write_str(&colored_str)
    }
}

impl Display for PartialConclusion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let colored_str = match self {
            PartialConclusion::INIT => format!(" {}", "∅".as_yellow()),
            PartialConclusion::SUCCESS => format!(" {}", "✓".as_green()),
            PartialConclusion::FAILED => format!(" {}", "✕".as_red()),
            PartialConclusion::CACHED => format!(" {}", "❤".as_blue()),
        };

        f.write_str(&colored_str)
    }
}

pub trait Colorize {
    fn as_black(&self) -> String;
    fn as_red(&self) -> String;
    fn as_green(&self) -> String;
    fn as_yellow(&self) -> String;
    fn as_blue(&self) -> String;
    fn as_purple(&self) -> String;
    fn as_turquoise(&self) -> String;
    fn as_gray(&self) -> String;
    fn as_white(&self) -> String;
}

#[derive(PartialEq, Clone)]
pub enum Color {
    Gray,
    Yellow,
    White,
    Blue,
    Purple,
    Turquoise,
    Red,
    Green,
    Black,
}

impl Color {
    pub fn get_index(i: usize) -> Color {
        let colors = Color::all();
        let index = i % colors.len();
        colors.get(index).unwrap().clone()
    }

    fn all() -> Vec<Color> {
        vec![
            Color::Gray,
            Color::Yellow,
            Color::White,
            Color::Blue,
            Color::Purple,
            Color::Turquoise,
            Color::Red,
            Color::Green,
            Color::Black,
        ]
    }
}

impl Colors for String {
    fn colorize(&self, color: &Color) -> String {
        self.as_str().colorize(color)
    }
}

impl Colors for &str {
    fn colorize(&self, color: &Color) -> String {
        match color {
            Color::Gray => self.as_gray(),
            Color::Yellow => self.as_yellow(),
            Color::White => self.as_white(),
            Color::Blue => self.as_blue(),
            Color::Purple => self.as_purple(),
            Color::Turquoise => self.as_turquoise(),
            Color::Red => self.as_red(),
            Color::Green => self.as_green(),
            Color::Black => self.as_black(),
        }
    }
}

pub trait Colors {
    fn colorize(&self, color: &Color) -> String;
}

impl Colorize for &str {
    fn as_black(&self) -> String { format!("\x1b[30m{self}\x1b[0m") }
    fn as_red(&self) -> String { format!("\x1b[31m{self}\x1b[0m") }
    fn as_green(&self) -> String { format!("\x1b[32m{self}\x1b[0m") }
    fn as_yellow(&self) -> String { format!("\x1b[33m{self}\x1b[0m") }
    fn as_blue(&self) -> String { format!("\x1b[34m{self}\x1b[0m") }
    fn as_purple(&self) -> String { format!("\x1b[35m{self}\x1b[0m") }
    fn as_turquoise(&self) -> String { format!("\x1b[36m{self}\x1b[0m") }
    fn as_gray(&self) -> String { format!("\x1b[37m{self}\x1b[0m") }
    fn as_white(&self) -> String { format!("\x1b[38m{self}\x1b[0m") }
}

impl Colorize for String {
    fn as_black(&self) -> String { format!("\x1b[30m{self}\x1b[0m") }
    fn as_red(&self) -> String { format!("\x1b[31m{self}\x1b[0m") }
    fn as_green(&self) -> String { format!("\x1b[32m{self}\x1b[0m") }
    fn as_yellow(&self) -> String { format!("\x1b[33m{self}\x1b[0m") }
    fn as_blue(&self) -> String { format!("\x1b[34m{self}\x1b[0m") }
    fn as_purple(&self) -> String { format!("\x1b[35m{self}\x1b[0m") }
    fn as_turquoise(&self) -> String { format!("\x1b[36m{self}\x1b[0m") }
    fn as_gray(&self) -> String { format!("\x1b[37m{self}\x1b[0m") }
    fn as_white(&self) -> String { format!("\x1b[38m{self}\x1b[0m") }
}
