use std::{env, fs, iter};
use std::path::{Path, PathBuf};

use anyhow::Context;
use filetime::FileTime;

pub fn resolve_executable(exec: &Path) -> anyhow::Result<PathBuf> {
    if exec.components().count() == 1 {
        let paths = env::var_os("PATH").ok_or_else(|| anyhow::format_err!("no PATH"))?;
        let candidates = env::split_paths(&paths).flat_map(|path| {
            let candidate = path.join(&exec);
            iter::once(candidate)
        });

        for candidate in candidates {
            if candidate.is_file() {
                return Ok(candidate);
            }
        }

        anyhow::bail!("no executable for `{}` found in PATH", exec.display())
    } else {
        Ok(exec.into())
    }
}

pub fn modification_time(path: &Path) -> anyhow::Result<FileTime> {
    let meta = fs::metadata(path).with_context(|| format!("failed to stat `{}`", path.display()))?;
    Ok(FileTime::from_last_modification_time(&meta))
}

pub fn read(path: &Path) -> anyhow::Result<String> {
    match String::from_utf8(read_bytes(path)?) {
        Ok(s) => Ok(s),
        Err(_) => anyhow::bail!("path at `{}` was not valid utf-8", path.display()),
    }
}

pub fn read_bytes(path: &Path) -> anyhow::Result<Vec<u8>> {
    fs::read(path).with_context(|| format!("failed to read `{}`", path.display()))
}

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> anyhow::Result<()> {
    let path = path.as_ref();
    fs::write(path, contents.as_ref()).with_context(|| format!("failed to write `{}`", path.display()))
}
