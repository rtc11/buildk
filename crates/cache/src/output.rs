use anyhow::Context;
use serde_derive::{Deserialize, Serialize};
use util::process_builder::ProcessBuilder;

use util::process_error::exit_status_to_string;

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Output {
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

    pub fn try_from(
        cmd: &ProcessBuilder,
        output: std::process::Output,
    ) -> anyhow::Result<Self> {
        let mut action = String::from(cmd.get_program().to_string_lossy());
        for arg in cmd.get_args() {
            action.push_str(&format!(" {}", arg.to_string_lossy()));
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| anyhow::anyhow!("{}: {:?}", e, e.as_bytes()))
            .with_context(|| "Failed to convert stdout to utf8")?;

        let stderr = String::from_utf8(output.stderr)
            .map_err(|e| anyhow::anyhow!("{}: {:?}", e, e.as_bytes()))
            .with_context(|| "Failed to convert stderr to utf8")?;

        let status = if output.status.success() { String::new() } else { exit_status_to_string(output.status) };

        Ok(Self {
            action,
            success: output.status.success(),
            status,
            code: output.status.code(),
            stdout,
            stderr
        })
    }
}
