pub mod toml;

use std::{fs, io};
use anyhow::Result;

pub fn rm(path: &String) -> Result<()> {
    let path = &std::path::PathBuf::from(&path);
    if path.is_file() {
        fs::remove_file(path)?
    } else {
        fs::remove_dir_all(path)?
    }

    Ok(())
}

#[allow(dead_code)]
pub fn ls(path: &String) -> Result<Vec<String>>{
    let entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path().into_os_string().into_string().unwrap()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    Ok(entries)
}
