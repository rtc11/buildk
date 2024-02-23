use std::fmt::Display;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Output {
    pub action: String,
    pub success: bool,
    pub status: String,
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    pub fn set_action(&mut self, action: String) {
        self.action = action
    }

    pub fn set_stdout(&mut self, stdout: String) {
        self.stdout = stdout
    }

    pub fn set_success(&mut self) {
        self.success = true
    }

}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.action)?;
        writeln!(f, "{}", self.success)?;
        writeln!(f, "{}", self.stdout)
    }
}
