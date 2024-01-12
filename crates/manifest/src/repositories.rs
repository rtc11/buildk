use std::str::FromStr;

use toml_edit::Document;

use crate::Section;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Repository {
    pub name: String,
    pub url: String,
}

impl Default for Repository {
    fn default() -> Self {
        Repository {
            name: "mavenCentral".into(),
            url: "https://repo1.maven.org/maven2".into(),
        }
    }
}

pub fn repositories(manifest: &Document) -> Vec<Repository> {
    let mut repos = vec![Repository::default()];

    let repos_from_manifest = manifest
        .as_table()
        .into_iter()
        .flat_map(|(key, value)| match Section::from_str(key) {
            Ok(Section::Repositories) => match value.as_table() {
                None => vec![],
                Some(table) => parse(table),
            },
            _ => vec![],
        })
        .collect::<Vec<Repository>>();

    repos.extend(repos_from_manifest);

    // uncomment to ensure distinct repos
    //repos.sort();
    //repos.dedup_by(|a, b| a.url == b.url);
    repos
}

fn parse(table: &toml_edit::Table) -> Vec<Repository> {
    table
        .iter()
        .map(|(name, url)| {
            Repository { 
                name: name.into(),
                url: url.to_string(), 
            }
        })
        .collect()
}
