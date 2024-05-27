use clap::{command, Parser, Subcommand, ValueEnum};

use build::Build;
use clean::Clean;
use dep_path::DepPath;
use deps::Deps;
use fetch::Fetch;
use init::Init;
use manifest::config::BuildK;
use process::{java::Java, kotlin::Kotlin, Process};
use release::Release;
use run::Run;
use test::Test;
use tree::Tree;
use util::buildk_output::BuildkOutput;

mod build;
mod clean;
mod config;
mod dep_path;
mod deps;
mod fetch;
mod init;
mod release;
mod run;
mod test;
mod tree;

#[derive(Parser)]
#[command(name = "")]
#[command(version = "0.1.0")]
#[command(about = "A Kotlin build tool for the 21st century")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short = 'q')]
    quiet: bool,
}

impl Cli {
    pub fn init() -> Cli {
        Cli::parse()
    }

    pub fn is_quiet(&self) -> bool {
        self.quiet
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build the project
    #[command(short_flag = 'b')]
    Build {
        #[arg(
        value_name = "SET",
        num_args = 0..=1,
        default_missing_value = "always",
        default_value_t = Set::All,
        value_enum
        )]
        set: Set,
    },

    /// Clean the output directory
    #[command(short_flag = 'c')]
    Clean {
        #[arg(
        value_name = "SET",
        num_args = 0..=1,
        default_missing_value = "always",
        default_value_t = CleanSet::All,
        value_enum
        )]
        set: CleanSet,
    },

    /// Show the project configuration
    Config,

    /// Print the dependencies
    Deps {
        #[arg(value_name = "LIMIT")]
        limit: Option<usize>,
    },

    /// Fetch the dependencies
    Fetch {
        #[arg(value_name = "ARTIFACT")]
        artifact: Option<String>,
    },

    /// Initialize the project
    Init, 

    /// Create a release (jar)
    Release,

    /// Run the project
    #[command(short_flag = 'r')]
    Run {
        #[arg(value_name = "MAIN")]
        name: Option<String>,
    },

    /// Run JUnit tests
    #[command(short_flag = 't')]
    Test {
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },

    /// Print the build tree
    Tree,

    Path {
        #[arg(value_name = "DEP")]
        dep: String,
    },
}

#[derive(ValueEnum, Copy, Clone, PartialEq, Eq)]
pub enum Set {
    All,
    Src,
    Test,
}

#[derive(ValueEnum, Copy, Clone, PartialEq, Eq)]
pub enum CleanSet {
    All,
    Src,
    Test,
    Release,
}

trait Command {
    type Item;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput;
}

impl Commands {
    pub fn apply(&mut self, buildk: &BuildK) -> BuildkOutput {
        let kotlin = Kotlin::new(buildk);
        let java = Java::new(buildk);
        let tree = Tree::new(buildk);

        match self {
            Commands::Build { set } => match kotlin {
                Ok(kotlin) => match tree {
                    Ok(tree) => Build::new(buildk, &kotlin, &tree).execute(Some(*set)),
                    Err(e) => panic!("{}", e),
                },
                Err(e) => panic!("{}", e),
            },
            Commands::Clean { set } => Clean::new(buildk).execute(Some(*set)),
            Commands::Config => config::Config::new(buildk).execute(None),
            Commands::Deps { limit } => Deps::new(buildk).execute(*limit),
            Commands::Fetch { artifact } => Fetch::new(buildk).execute(artifact.clone()),
            Commands::Init => Init::new().execute(None),
            Commands::Release => match kotlin {
                Ok(kotlin) => Release::new(buildk, &kotlin).execute(None),
                Err(e) => panic!("{}", e),
            },
            Commands::Run { name } => match java {
                Ok(java) => Run::new(buildk, &java).execute(name.clone()),
                Err(e) => panic!("{}", e),
            },
            Commands::Test { name } => match java {
                Ok(java) => Test::new(buildk, &java).execute(name.clone()),
                Err(e) => panic!("{}", e),
            },
            Commands::Tree => match tree {
                Ok(mut tree) => tree.execute(None),
                Err(e) => panic!("{}", e),
            },
            Commands::Path { dep } => DepPath::new(buildk).execute(Some(dep.to_owned())),
        }
    }
}
