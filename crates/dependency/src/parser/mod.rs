use std::{collections::BTreeSet, path::Path};

use crate::{Package, PackageKind, Parser};

pub mod buildk;
pub mod gradle;
pub mod maven;

pub fn parse(path: &Path, kind: PackageKind) -> BTreeSet<Package> {
    let descriptor = path.join("maven.xml");
    if descriptor.exists() {
        return maven::MavenParser::parse(descriptor, kind);
    }

    let descriptor = path.join("gradle.json");
    if descriptor.exists() {
        return gradle::GradleParser::parse(descriptor, kind);
    }

    return BTreeSet::new();
}
