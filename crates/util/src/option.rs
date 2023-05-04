use std::fmt::{Display, Formatter};

pub enum Option {
    Clean,
    BuildTest,
    BuildSrc,
    Fetch,
    Test,
    Run,
    Release,
    List,
    Help,
}

impl Option {
    pub fn from(value: String) -> Vec<Option> {
        match value.as_str() {
            "clean" => vec![Option::Clean],
            "fetch" => vec![Option::Fetch],
            "build" => vec![Option::Fetch, Option::BuildSrc, Option::BuildTest],
            "test" => vec![Option::Fetch, Option::BuildSrc, Option::BuildTest, Option::Test],
            "run" => vec![Option::Fetch, Option::BuildSrc, Option::Run],
            "release" => vec![Option::Fetch, Option::BuildSrc, Option::Release],
            "list" => vec![Option::List],
            _ => vec![]
        }
    }
}

impl Display for Option {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Option::Clean => "clean",
            Option::BuildSrc => "build src",
            Option::BuildTest => "build test",
            Option::Fetch => "fetch",
            Option::Test => "test",
            Option::Run => "run",
            Option::Release => "release",
            Option::List => "list",
            Option::Help => "help",

        };

        write!(f, "{:<12}", display)
    }
}
