use anyhow::Result;

use crate::PROJECT;

pub(crate) fn build_dir() -> Result<String> {
    fs::rm(&std::path::PathBuf::from(format!("{}/build", PROJECT)))?;
    Ok(String::default())
}
