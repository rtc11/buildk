use anyhow::Context;
use serde_derive::{Deserialize, Serialize};

use util::process_error::exit_status_to_string;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Output {
    pub success: bool,
    pub status: String,
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl TryInto<Output> for std::process::Output {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Output, Self::Error> {
        let stdout = String::from_utf8(self.stdout)
            .map_err(|e| anyhow::anyhow!("{}: {:?}", e, e.as_bytes()))
            .with_context(|| "Failed to convert stdout to utf8")?;

        let stderr = String::from_utf8(self.stderr)
            .map_err(|e| anyhow::anyhow!("{}: {:?}", e, e.as_bytes()))
            .with_context(|| "Failed to convert stderr to utf8")?;

        Ok(Output {
            success: self.status.success(),
            status: if self.status.success() {
                String::new()
            } else {
                exit_status_to_string(self.status)
            },
            code: self.status.code(),
            stdout,
            stderr,
        })
    }
}
