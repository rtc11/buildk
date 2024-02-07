use std::path::PathBuf;

use colorize::Colorize;
use terminal::Printable;

pub mod buildk_output;
pub mod colorize;
pub mod hasher;
pub mod paths;
pub mod process_builder;
pub mod process_error;
pub mod terminal;
pub mod timer;

pub enum Conclusion {
    SUCCESS,
    FAILED,
}

impl Printable for Conclusion {
    fn print(&self, terminal: &mut terminal::Terminal) {
        let colored_str = match self {
            Conclusion::SUCCESS => "SUCCESS".as_green(),
            Conclusion::FAILED => "FAILED".as_red(),
        };

        terminal.print(&colored_str);
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum PartialConclusion {
    INIT,
    CACHED,
    SUCCESS,
    FAILED,
}

impl PartialConclusion {
    pub fn color_symbol(&self) -> String {
        match self {
            PartialConclusion::INIT => format!(" {}", "∅".as_yellow()),
            PartialConclusion::SUCCESS => format!(" {}", "✓".as_green()),
            PartialConclusion::FAILED => format!(" {}", "✕".as_red()),
            PartialConclusion::CACHED => format!(" {}", "❤".as_blue()),
        }
    }
}

impl Printable for PartialConclusion {
    fn print(&self, terminal: &mut terminal::Terminal) {
        let colored_str = match self {
            PartialConclusion::INIT => format!(" {}", "∅".as_yellow()),
            PartialConclusion::SUCCESS => format!(" {}", "✓".as_green()),
            PartialConclusion::FAILED => format!(" {}", "✕".as_red()),
            PartialConclusion::CACHED => format!(" {}", "❤".as_blue()),
        };

        terminal.print(&colored_str);
    }
}

pub fn get_kotlinc() -> PathBuf {
    get_kotlin_home().join("bin/kotlinc")
}

pub fn get_kotlin_home() -> PathBuf {
    match option_env!("KOTLIN_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from("/usr/local/Cellar/kotlin/1.9.22/"),
    }
}

pub trait StrExtras {
    fn replace_after_last(&self, pattern: &str) -> &str;
    fn replace_before_last(&self, pattern: &str) -> &str;
    fn replace_after(&self, pattern: &str) -> &str;
    fn replace_before(&self, pattern: &str) -> &str;
}

pub trait StringExtras {
    fn replace_after_last(&self, pattern: &str) -> String;
    fn replace_before_last(&self, pattern: &str) -> String;
    fn replace_after(&self, pattern: &str) -> String;
    fn replace_before(&self, pattern: &str) -> String;
}

impl StrExtras for &str {
    fn replace_after_last(&self, pattern: &str) -> &str {
        let haystack = self;
        if let Some(last_pos) = haystack.rfind(pattern) {
            &haystack[..last_pos]
        } else {
            haystack
        }
    }

    fn replace_before_last(&self, pattern: &str) -> &str {
        let haystack = self;
        if let Some(last_pos) = haystack.rfind(pattern) {
            &haystack[last_pos..]
        } else {
            haystack
        }
    }

    fn replace_after(&self, pattern: &str) -> &str {
        let haystack = self;
        if let Some(first_pos) = haystack.find(pattern) {
            &haystack[first_pos..]
        } else {
            haystack
        }
    }

    fn replace_before(&self, pattern: &str) -> &str {
        let haystack = self;
        if let Some(first_pos) = haystack.find(pattern) {
            &haystack[..first_pos]
        } else {
            haystack
        }
    }
}

impl StringExtras for String {
    fn replace_after_last(&self, pattern: &str) -> String {
        let haystack = self;
        if let Some(last_pos) = haystack.rfind(pattern) {
            haystack[..last_pos].to_string()
        } else {
            haystack.to_string()
        }
    }

    fn replace_before_last(&self, pattern: &str) -> String {
        let haystack = self;
        if let Some(last_pos) = haystack.rfind(pattern) {
            haystack[last_pos..].to_string()
        } else {
            haystack.to_string()
        }
    }

    fn replace_after(&self, pattern: &str) -> String {
        let haystack = self;
        if let Some(first_pos) = haystack.find(pattern) {
            haystack[first_pos..].to_string()
        } else {
            haystack.to_string()
        }
    }

    fn replace_before(&self, pattern: &str) -> String {
        let haystack = self;
        if let Some(first_pos) = haystack.find(pattern) {
            haystack[..first_pos].to_string()
        } else {
            haystack.to_string()
        }
    }
}
