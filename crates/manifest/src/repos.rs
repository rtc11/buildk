use std::{fmt::Display, str::FromStr};

use toml_edit::DocumentMut;

use crate::Section;

#[derive(Clone)]
pub struct Repos {
    pub repos: Vec<Repo>,
}

impl From<DocumentMut> for Repos {
    fn from(value: DocumentMut) -> Self {
        let mut repos = vec![Repo::default()];

        let repos_from_manifest = value
            .as_table()
            .into_iter()
            .flat_map(|(key, value)| match Section::from_str(key) {
                Ok(Section::Repos) => match value.as_table() {
                    None => vec![],
                    Some(table) => parse(table),
                },
                _ => vec![],
            })
            .collect::<Vec<_>>();

        repos.extend(repos_from_manifest);
        Repos { repos }
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Repo {
    pub name: String,
    pub url: String,
}

impl Default for Repo {
    fn default() -> Self {
        Repo {
            name: "mavenCentral".into(),
            url: "https://repo1.maven.org/maven2".into(),
        }
    }
}

impl Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "repo", self.url)
    }
}

fn parse(table: &toml_edit::Table) -> Vec<Repo> {
    table
        .iter()
        .map(|(name, url)| Repo {
            name: name.into(),
            url: url.to_string().replace("\"", "").replace(" ", ""),
        })
        .collect()
}
