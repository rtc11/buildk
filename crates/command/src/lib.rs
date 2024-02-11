use std::path::PathBuf;

use anyhow::Result;
use build::Build;
use cache::cache::Cache;
use clap::{Parser, Subcommand, ValueEnum, command};
use clean::Clean;
use deps::Deps;
use fetch::Fetch;
use ::manifest::config::Config;
use http::client::Client;
use release::Release;
use run::Run;
use test::Test;
use tree::Tree;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;
use util::get_kotlin_home;

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
#[command(name = "buildk")]
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
        let mut cache = Cache::new(config);
        let mut tree = Tree::new(config);

        match self {
            Commands::Build { set } => Build::new(config, &mut cache, &tree).execute(Some(*set)),
            Commands::Clean => Clean::new(config, &mut cache).execute(None),
            Commands::Config => config::Config::new(config).execute(None),
            Commands::Deps => Deps::new(config, &mut cache).execute(None),
            Commands::Fetch => Fetch::new(config, &mut cache).execute(None),
            Commands::Release => Release::new(config, &mut cache).execute(None),
            Commands::Run { name } => Run::new(config, &mut cache).execute(name.clone()),
            Commands::Test { name } => Test::new(config, &mut cache).execute(name.clone()),
            Commands::Tree => tree.execute(None),
        }
    }
}

pub struct KotlinCompiler {
    pub version: String,
    pub _test_libs: Vec<PathBuf>,
    pub client: Client,
}

impl KotlinCompiler {
    pub fn new(config: &Config) -> Result<KotlinCompiler> {
        let kotlin_home = get_kotlin_home();
        let mut cache = Cache::new(config);

        let mut kotlinc = KotlinCompiler {
            version: "unknown".to_string(),
            _test_libs: vec![
                kotlin_home.join("libexec/lib/kotlin-test-junit5.jar"),
                kotlin_home.join("libexec/lib/kotlin-test.jar"),
            ],
            client: Client
        };

        let mut runner = ProcessBuilder::new(kotlin_home.join("bin/kotlin"));
        runner.cwd(&config.manifest.project.path).arg("-version");

        let cache_res = cache.cache_command(&runner, 0)?;
        let version = cache_res
            .stdout
            .expect("kotlinc -version gave no stdout")
            .lines()
            .find(|l| l.starts_with("Kotlin version "))
            .map(|l| l.replace("Kotlin version ", ""))
            .ok_or_else(|| {
                anyhow::format_err!("`kotlinc -version` didnt have a line for `Kotlin version")
            })?;

        kotlinc.version = version;

        Ok(kotlinc)
    }

}
