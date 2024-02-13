use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use anyhow::ensure;
use manifest::dependencies::Dependency;

use paths::modification_time;
use util::paths;
use util::hasher::StableHasher;

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

pub fn dependency_fingerprint(dependency: &Dependency) -> anyhow::Result<u64> {
    let mut hasher = StableHasher::default();
    ensure!(dependency.target_dir.is_dir());
    dependency.target_dir.hash(&mut hasher);
    modification_time(&dependency.target_dir)?.hash(&mut hasher);
    Ok(hasher.finish())
}
