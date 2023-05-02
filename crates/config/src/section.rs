use std::str::FromStr;

pub enum Section {
    Project,
    Module,
    Dependencies,
    TestDependencies,
}

impl FromStr for Section {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "project" => Section::Project,
            "module" => Section::Module,
            "dependencies" => Section::Dependencies,
            "test-dependencies" => Section::TestDependencies,
            _ => anyhow::bail!("Invalid section: {}", s),
        })
    }
}