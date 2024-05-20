#![allow(dead_code)]

use std::{collections::BTreeSet, fs, path::PathBuf};

use crate::{Dependency, Parser};
pub struct GradleParser;

impl Parser<crate::Dependency> for GradleParser {
    fn parse(path: PathBuf) -> BTreeSet<crate::Dependency> {
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

        descriptor.into()
    }
}

impl From<gradle::Descriptor> for BTreeSet<crate::Dependency> {
    fn from(value: gradle::Descriptor) -> Self {
        let mut deps = BTreeSet::new();

        if let Some(component) = value.component {
            deps.insert(Dependency::from(component));
        }

        for variant in value.variants {
            if let Some(dependencies) = variant.dependencies {
                for dep in dependencies {
                    deps.insert(Dependency::from(dep));
                }
            }
        }

        deps
    }
}

/*
* Gradle module metadata specification v1.1
* ref: https://github.com/gradle/gradle/blob/master/platforms/documentation/docs/src/docs/design/gradle-module-metadata-latest-specification.md
*/
mod gradle {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

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

    impl From<Component> for crate::Dependency {
        fn from(value: Component) -> Self {
            crate::Dependency::new(
                value.group,
                value.module,
                value.version,
                crate::Kind::Compile,
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

    impl From<Dep> for crate::Dependency {
        fn from(value: Dep) -> Self {
            crate::Dependency::new(
                value.group,
                value.module,
                value.version.into(),
                crate::Kind::Compile,
            )
        }
    }

    impl From<Version> for String {
        fn from(value: Version) -> Self {

            // if single required, use it
            if let Some(val) = &value.requires {
                match val {
                    VersionKind::Single(val) => return val.to_owned(),
                    _ => {}
                }
            }

            if let Some(val) = value.prefers {
                return match val {
                    VersionKind::Single(val) => val,
                    VersionKind::Array(val) => val[0].clone(),
                }
            }

            /*
            if let Some(val) = value.requires {
                return val;
            }

            if let Some(val) = value.prefers {
                return val;
            }
            if let Some(val) = value.strictly {
                return val;
            }
            */

            panic!("unable to determine version at this time. Found following: {:?}", &value)
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

        let dependencies: BTreeSet<crate::Dependency> = descriptor.into();

        assert_eq!(dependencies.len(), 3);
    }
}
