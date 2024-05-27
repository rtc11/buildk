use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use anyhow::ensure;

use dependency::Package;
use paths::modification_time;
use util::hasher::StableHasher;
use util::paths;

pub mod cache;
mod data;
pub mod output;


pub fn file_fingerprint(path: &PathBuf) -> anyhow::Result<u64> {
    let mut hasher = StableHasher::default();
    ensure!(path.is_file());
    path.hash(&mut hasher);
    modification_time(path)?.hash(&mut hasher);
    Ok(hasher.finish())
}

pub fn dependency_fingerprint(pkg: &Package) -> anyhow::Result<u64> {
    let mut hasher = StableHasher::default();
    ensure!(pkg.location.is_dir());
    pkg.location.hash(&mut hasher);
    modification_time(&pkg.location)?.hash(&mut hasher);
    Ok(hasher.finish())
}
