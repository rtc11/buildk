use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct Classpath {
    src: Arc<Mutex<HashSet<PathBuf>>>,
    test: Arc<Mutex<HashSet<PathBuf>>>,
}

impl Classpath {
    pub fn add_src(&mut self, path: &Path) {
        self.src.lock().unwrap().insert(path.to_owned());
    }

    pub fn add_test(&mut self, path: &Path) {
        self.test.lock().unwrap().insert(path.to_owned());
    }

    pub fn extend_test(&mut self, paths: Vec<&PathBuf>) {
        self.test.lock().unwrap().extend(paths.into_iter().cloned().collect::<Vec<_>>())
    }

    pub fn extend_src(&mut self, paths: Vec<&PathBuf>) {
        self.src.lock().unwrap().extend(paths.into_iter().cloned().collect::<Vec<_>>())
    }

    pub fn get_src(&self) -> Vec<PathBuf> {
        self.src.lock().unwrap().clone().into_iter().collect()
    }

    pub fn get_test(&self) -> Vec<PathBuf> {
        self.test.lock().unwrap().clone().into_iter().collect()
    }
}
