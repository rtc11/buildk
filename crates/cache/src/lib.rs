use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use anyhow::ensure;

use paths::{modification_time, resolve_executable};
use util::{BuildkResult, get_kotlinc, paths};
use util::hasher::StableHasher;
use util::process_builder::ProcessBuilder;

pub mod cache;
mod data;
mod output;

fn kotlinc_fingerprint(kotlin_bin: &Path) -> BuildkResult<u64> {
    let kotlinc = kotlin_bin.join(get_kotlinc());
    let mut hasher = StableHasher::default();
    let hash_exe = |hasher: &mut _, path| -> BuildkResult<()> {
        let path = resolve_executable(path)?;
        path.hash(hasher);
        modification_time(&path)?.hash(hasher);
        Ok(())
    };

    hash_exe(&mut hasher, &kotlinc)?;
    Ok(hasher.finish())
}

fn process_fingerprint(cmd: &ProcessBuilder, extra_fingerprint: u64) -> u64 {
    let mut hasher = StableHasher::default();
    extra_fingerprint.hash(&mut hasher);
    cmd.get_args().for_each(|arg| arg.hash(&mut hasher));
    let mut env = cmd.get_envs().iter().collect::<Vec<_>>();
    env.sort_unstable();
    env.hash(&mut hasher);
    hasher.finish()
}

fn kt_fingerprint(path: &PathBuf) -> anyhow::Result<u64> {
    let mut hasher = StableHasher::default();
    ensure!(path.is_file());
    path.hash(&mut hasher);
    modification_time(path)?.hash(&mut hasher);
    Ok(hasher.finish())
}
