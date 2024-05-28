use std::{collections::BTreeSet, path::Path};

use crate::{Package, Parser};

pub mod buildk;
pub mod gradle;
pub mod maven;

pub fn parse(path: &Path) -> BTreeSet<Package> {
    let descriptor = path.join("maven.xml");
    if descriptor.exists() {
        return maven::MavenParser::parse(descriptor);
    }

    let descriptor = path.join("gradle.json");
    if descriptor.exists() {
        return gradle::GradleParser::parse(descriptor);
    }

    return BTreeSet::new();
}
