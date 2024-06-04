#![allow(dead_code)]

use std::{collections::BTreeSet, fs, path::PathBuf};

use crate::{Package, PackageKind, Parser};
pub struct GradleParser;

#[derive(Ord, PartialOrd, Eq, PartialEq, Default, Debug)]
pub struct GradlePackage {
    group: String,
    module: String,
    version: String,
    versions: Vec<String>, // sometimes defined with range, e.g. [1.0, 2.0)
    variant: GradleVariant,
    location: PathBuf,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Default, Debug)]
pub enum GradleVariant {
    #[default]
    Implementation,
    TestImplementation,
    Api,
    Runtime,
}

// https://docs.gradle.org/current/userguide/dependency_management.html#sec:how-gradle-downloads-deps
impl Parser<Package> for GradleParser {
    fn parse(path: PathBuf, kind: PackageKind) -> BTreeSet<Package> {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => return BTreeSet::default(),
        };

        let descriptor: gradle::Descriptor = match serde_json::from_str(&content) {
            Ok(descriptor) => descriptor,
            Err(err) => {
                eprintln!("unable to parse gradle .module file: {}", err);
                eprintln!("content: {}", content);
                return BTreeSet::default();
            }
        };

        if descriptor.format_version != "1.1" {
            eprintln!("currently only support gradle module metadata spec v1.1");
            return BTreeSet::default();
        };

        let packages: BTreeSet<Package> = descriptor.into();
        packages.into_iter().filter(|it| kind == it.kind).collect()
    }
}

impl From<gradle::Descriptor> for BTreeSet<Package> {
    fn from(value: gradle::Descriptor) -> Self {
        let mut deps = BTreeSet::new();

        // self
        if let Some(component) = value.component {
            deps.insert(component.into());
        }

        // dependencies
        for variant in value.variants {
            if let Some(dependencies) = variant.dependencies {
                for dep in dependencies {
                    deps.insert(dep.into());
                }
            }
        }

        deps
    }
}

impl From<GradlePackage> for Package {
    fn from(value: GradlePackage) -> Self {
        Package::new(
            value.module,
            Some(value.group),
            value.version,
            match value.variant {
                GradleVariant::Api => PackageKind::Compile,
                GradleVariant::Runtime => PackageKind::Runtime,
                GradleVariant::Implementation => PackageKind::Compile,
                GradleVariant::TestImplementation => PackageKind::Test,
            },
        )
    }
}

/*
* Gradle module metadata specification v1.1
* ref: https://github.com/gradle/gradle/blob/master/platforms/documentation/docs/src/docs/design/gradle-module-metadata-latest-specification.md
*/
mod gradle {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    use crate::Package;

    #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Descriptor {
        pub format_version: String,
        pub component: Option<Component>,
        pub created_by: Option<CreatedBy>,
        pub variants: Vec<Variant>,
    }

    #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
    pub(crate) struct Component {
        group: String,
        module: String,
        version: String,
        url: Option<String>, // URL to the component's metadata file
    }

    impl From<Component> for Package {
        fn from(value: Component) -> Self {
            Package::new(
                value.module,
                Some(value.group),
                value.version,
                Default::default(), // TODO: resolve variant
            )
        }
    }

    #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
    pub(crate) struct CreatedBy {
        gradle: Option<Gradle>,
    }

    #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Gradle {
        version: String,
        build_id: Option<String>,
    }

    #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Variant {
        name: String,
        attributes: Option<BTreeMap<String, AttributeValue>>,
        available_at: Option<AvailableAt>,  // available-at
        pub dependencies: Option<Vec<Dep>>, //  must be present when available-at is present
        dependency_constraints: Option<Vec<DependencyConstraint>>,
        files: Option<Vec<File>>,
        capabilities: Option<Vec<Capability>>,
    }

    #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    pub(crate) enum AttributeValue {
        String(String),
        Number(u64),
    }

    #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
    pub(crate) struct DependencyConstraint {
        group: String,
        module: String,
        version: Option<Version>,
        reason: Option<String>,
        attributes: Option<BTreeMap<String, String>>,
    }

    #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
    pub(crate) struct Capability {
        group: String,
        module: String,
        version: String,
    }

    #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
    pub(crate) struct AvailableAt {
        url: String,
        group: String,
        module: String,
        version: String,
    }

    #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Dep {
        group: String,
        module: String,
        version: Version,
        excludes: Option<Vec<Exclusion>>,
        attributes: Option<BTreeMap<String, String>>,
        requested_capabilities: Option<Vec<Capability>>,
    }

    #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
    pub(crate) struct Exclusion {
        group: String,
        module: String,
    }

    impl From<Dep> for Package {
        fn from(value: Dep) -> Self {
            Package::new(
                value.module.clone(),
                Some(value.group.clone()),
                value.version().clone(),
                Default::default(), // TODO: resolve variant
            )
        }
    }

    impl Dep {
        fn version(&self) -> String {
            if let Some(version) = &self.version.requires {
                match version {
                    VersionKind::Single(version) => return version.to_owned(),
                    VersionKind::Array(versions) => {
                        // TODO: skip last while self.version.rejects.contains(last)
                        if let Some(last) = versions.last() {
                            return last.to_owned();
                        }
                    }
                }
            }

            if let Some(version) = &self.version.prefers {
                match version {
                    VersionKind::Single(version) => return version.to_owned(),
                    VersionKind::Array(versions) => {
                        // TODO: skip last while self.version.rejects.contains(last)
                        if let Some(last) = versions.last() {
                            return last.to_owned();
                        }
                    }
                }
            }

            if let Some(version) = &self.version.strictly {
                match version {
                    VersionKind::Single(version) => return version.to_owned(),
                    VersionKind::Array(versions) => {
                        // TODO: skip last while self.version.rejects.contains(last)
                        if let Some(last) = versions.last() {
                            return last.to_owned();
                        }
                    }
                }
            }

            panic!("unable to determine version at this time")
        }

        fn versions(&self) -> Vec<String> {
            if let Some(version) = &self.version.requires {
                if let VersionKind::Array(versions) = version {
                    return versions.to_owned();
                }
            }

            if let Some(version) = &self.version.prefers {
                if let VersionKind::Array(versions) = version {
                    return versions.to_owned();
                }
            }
            if let Some(version) = &self.version.strictly {
                if let VersionKind::Array(versions) = version {
                    return versions.to_owned();
                }
            }

            vec![]
        }
    }

    #[derive(Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
    pub(crate) struct Version {
        requires: Option<VersionKind>,
        prefers: Option<VersionKind>,
        strictly: Option<VersionKind>,
        rejects: Option<VersionKind>,
    }

    #[derive(Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
    #[serde(untagged)]
    pub(crate) enum VersionKind {
        Single(String), // "1.0" or "[1.0, 2.0)"]
        Array(Vec<String>),
    }

    #[derive(Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq)]
    pub(crate) struct File {
        name: String,
        url: String,
        size: u64,
        sha1: String,
        sha256: Option<String>,
        sha512: Option<String>,
        md5: String,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::Package;

    use super::gradle::Descriptor;

    #[test]
    fn test_example() {
        let content = r#"
{
    "formatVersion": "1.0",
    "component": {
        "group": "my.group",
        "module": "mylib",
        "version": "1.2"
    },
    "createdBy": {
        "gradle": {
            "version": "4.3",
            "buildId": "abc123"
        }
    },
    "variants": [
        {
            "name": "api",
            "attributes": {
                "org.gradle.usage": "java-api",
                "org.gradle.category": "library",
                "org.gradle.libraryelements": "jar"
            },
            "files": [
                { 
                    "name": "mylib-api.jar", 
                    "url": "mylib-api-1.2.jar",
                    "size": 1453,
                    "sha1": "abc12345",
                    "md5": "abc12345"
                }
            ],
            "dependencies": [
                { 
                    "group": "some.group", 
                    "module": "other-lib", 
                    "version": { "requires": "3.4" },
                    "excludes": [
                        { "group": "*", "module": "excluded-lib" }
                    ],
                    "attributes": {
                       "buildType": "debug"
                    }
                }
            ]
        },
        {
            "name": "runtime",
            "attributes": {
                "org.gradle.usage": "java-runtime",
                "org.gradle.category": "library",
                "org.gradle.libraryelements": "jar"
            },
            "files": [
                { 
                    "name": "mylib.jar", 
                    "url": "mylib-1.2.jar",
                    "size": 4561,
                    "sha1": "abc12345",
                    "md5": "abc12345"
                }
            ],
            "dependencies": [
                { 
                    "group": "some.group", 
                    "module": "other-lib", 
                    "version": { "requires": "[3.0, 4.0)", "prefers": "3.4", "rejects": ["3.4.1"] } 
                }
            ],
            "dependencyConstraints": [
                { 
                    "group": "some.group", 
                    "module": "other-lib-2", 
                    "version": { "requires": "1.0" } 
                }
            ]
        }
    ]
}
        "#;

        let descriptor: Descriptor =
            serde_json::from_str(content).expect("unable to parse gradle module descriptor");

        let dependencies: BTreeSet<Package> = descriptor.into();

        assert_eq!(dependencies.len(), 3);
    }
}
