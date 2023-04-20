pub mod toml;

use std::fs::{
    // copy,
    // create_dir,
    // read_dir,
    remove_dir_all,
    remove_file
};
use std::io::Result;
use std::path::PathBuf;

pub fn rm(path: &PathBuf) -> Result<()> {
    if path.is_file() {
        remove_file(path)?
    } else {
        remove_dir_all(path)?
    }

    Ok(())
}
