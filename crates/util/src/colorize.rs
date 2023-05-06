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

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OrderedColor {
    Yellow,
    Gray,
    White,
    Blue,
    Purple,
    Turquoise,
    Red,
    Green,
    Black,
}

pub struct ColorRoulette {
    current: Option<OrderedColor>,
    colors: Vec<OrderedColor>,
}


impl Default for ColorRoulette {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorRoulette {
    fn new() -> ColorRoulette {
        Self {
            current: None,
            colors: OrderedColor::all(),
        }
    }

    pub fn next_color(&mut self) -> OrderedColor {
        match self.current {
            None => {
                self.current = Some(OrderedColor::Yellow);
                self.current.clone().unwrap()
            },
            Some(OrderedColor::Black) => {
                self.current = Some(OrderedColor::White);
                self.current.clone().unwrap()
            }
            _ => {
                let (index, _) = self.colors.iter().enumerate().find(|(_, it)| {
                    it == &&self.current.clone().unwrap()
                }).unwrap();
                let color = self.colors.get(index + 1).unwrap().clone();
                self.current = Some(color.clone());
                color
            }
        }
    }
}

impl OrderedColor {
    pub fn all() -> Vec<OrderedColor> {
        vec![
            OrderedColor::Yellow,
            OrderedColor::Gray,
            OrderedColor::White,
            OrderedColor::Blue,
            OrderedColor::Purple,
            OrderedColor::Turquoise,
            OrderedColor::Red,
            OrderedColor::Green,
            OrderedColor::Black,
        ]
    }
}

impl Colors for String {
    fn colorize(&self, color: &OrderedColor) -> String {
        self.as_str().colorize(color)
    }
}

impl Colors for &str {
    fn colorize(&self, color: &OrderedColor) -> String {
        match color {
            OrderedColor::Yellow => self.as_yellow(),
            OrderedColor::Gray => self.as_gray(),
            OrderedColor::White => self.as_white(),
            OrderedColor::Blue => self.as_blue(),
            OrderedColor::Purple => self.as_purple(),
            OrderedColor::Turquoise => self.as_turquoise(),
            OrderedColor::Red => self.as_red(),
            OrderedColor::Green => self.as_green(),
            OrderedColor::Black => self.as_black(),
        }
    }
}

pub trait Colors {
    fn colorize(&self, color: &OrderedColor) -> String;
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
