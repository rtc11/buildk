use std::collections::BTreeSet;
use std::path::PathBuf;

mod buildk_parser;
mod gradle_parser;
pub mod maven_parser;

pub trait Parser<T> where T: Ord {
    fn parse(pom: PathBuf) -> BTreeSet<T>;
}
