#![allow(dead_code)]
use std::path::PathBuf;
use crate::dependencies::Dependency;

pub struct Module {
    path: PathBuf,
    dependencies: Vec<Dependency>
}
