use ::manifest::config::Config;
use build::Build;
use cache::cache::Cache;
use clap::{command, Parser, Subcommand, ValueEnum};
use clean::Clean;
use deps::Deps;
use fetch::Fetch;
use process::{kotlin::Kotlin, Process, java::Java};
use release::Release;
use run::Run;
use test::Test;
use tree::Tree;
use util::buildk_output::BuildkOutput;

mod build;
mod clean;
mod config;
mod deps;
mod fetch;
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
    command: Commands,
}

impl Cli {
    pub fn commands() -> Commands {
        Cli::parse().command
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
    Clean,

    /// Show the project configuration
    Config,

    /// Print the dependencies
    Deps,

    /// Fetch the dependencies
    Fetch,

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
}

#[derive(ValueEnum, Copy, Clone, PartialEq, Eq)]
pub enum Set {
    All,
    Src,
    Test,
}

trait Command {
    type Item;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput;
}

impl Commands {
    pub fn apply(&mut self, config: &Config) -> BuildkOutput {
        let mut cache = Cache::load(&config.manifest.project.out.cache);
        let mut tree = Tree::new(config);
        let kotlin = Kotlin::new(config).expect("kotlin not found");
        let java = Java::new(config).expect("java not found");

        match self {
            Commands::Build { set } => Build::new(config, &kotlin, &mut cache, &tree).execute(Some(*set)),
            Commands::Clean => Clean::new(config, &mut cache).execute(None),
            Commands::Config => config::Config::new(config).execute(None),
            Commands::Deps => Deps::new(config, &mut cache).execute(None),
            Commands::Fetch => Fetch::new(config, &cache).execute(None),
            Commands::Release => Release::new(config, &kotlin).execute(None),
            Commands::Run { name } => Run::new(config, &java).execute(name.clone()),
            Commands::Test { name } => Test::new(config, &java).execute(name.clone()),
            Commands::Tree => tree.execute(None),
        }
    }
}

