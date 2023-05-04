use std::fmt::{Display, Formatter};

pub enum Cmd {
    Clean,
    BuildTest,
    BuildSrc,
    Fetch,
    Test,
    Run,
    Release,
}

impl Cmd {
    pub fn from(value: String) -> Vec<Cmd> {
        match value.as_str() {
            "clean" => vec![Cmd::Clean],
            "fetch" => vec![Cmd::Fetch],
            "build" => vec![Cmd::Fetch, Cmd::BuildSrc, Cmd::BuildTest],
            "test" => vec![Cmd::Fetch, Cmd::BuildSrc, Cmd::BuildTest, Cmd::Test],
            "run" => vec![Cmd::Fetch, Cmd::BuildSrc, Cmd::Run],
            "release" => vec![Cmd::Fetch, Cmd::BuildSrc, Cmd::Release],
            _ => vec![]
        }
    }
}

impl Display for Cmd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Cmd::Clean => "clean",
            Cmd::BuildSrc => "build src",
            Cmd::BuildTest => "build test",
            Cmd::Fetch => "fetch",
            Cmd::Test => "test",
            Cmd::Run => "run",
            Cmd::Release => "release",
        };

        write!(f, "{:<12}", display)
    }
}
