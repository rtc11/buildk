use anyhow::Result;
use filetime::FileTime;
use std::{
    env, fs, iter,
    path::{Path, PathBuf},
};

pub fn resolve_executable(exec: &Path) -> Result<PathBuf> {
    if exec.components().count() == 1 {
        let paths = env::var_os("PATH").ok_or_else(|| anyhow::format_err!("no PATH"))?;
        let candidates = env::split_paths(&paths).flat_map(|path| {
            let candidate = path.join(exec);
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

pub fn modification_time(path: &Path) -> Result<FileTime> {
    let meta = fs::metadata(path)?;
    Ok(FileTime::from_last_modification_time(&meta))
}

pub fn read(path: &Path) -> Result<String> {
    let bytes = read_bytes(path)?;
    let string = String::from_utf8(bytes)?;
    Ok(string)
}

pub fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    let bytes = fs::read(path)?;
    Ok(bytes)
}

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    fs::write(&path, &contents)?;
    Ok(())
}

//#[async_recursion]
pub fn all_files_recursive(mut files: Vec<PathBuf>, path: PathBuf) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        files.push(path)
    } else if path.is_dir() {
        let dir = std::fs::read_dir(&path)?;

        for res in dir.flatten() {
            let next_files = all_files_recursive(vec![], res.path());
            files.extend(next_files?);
        }
    }
    Ok(files)
}
