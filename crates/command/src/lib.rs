use anyhow::Result;
use cache::cache::Cache;
use clap::{Parser, Subcommand, ValueEnum};
use ::manifest::config::Config;
use http::client::Client;
use util::buildk_output::BuildkOutput;
use util::colorize::Colorize;
use util::process_builder::ProcessBuilder;
use util::{get_kotlin_home, PartialConclusion};

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

trait BuildCmd {
    fn build(&mut self, config: &Config, set: Set) -> BuildkOutput;
}

trait CleanCmd {
    fn clean(&mut self, config: &Config) -> BuildkOutput;
}

trait ConfigCmd {
    fn config(&mut self, config: &Config) -> BuildkOutput;
}

trait DepsCmd {
    fn deps(&mut self, config: &Config) -> BuildkOutput;
}

trait FetchCmd {
    fn fetch(&mut self, config: &Config) -> BuildkOutput;
}

trait HelpCmd {
    fn help(&mut self, config: &Config) -> BuildkOutput;
}

trait ReleaseCmd {
    fn release(&mut self, config: &Config) -> BuildkOutput;
}

trait RunCmd {
    fn run(&mut self, config: &Config, name: Option<String>) -> BuildkOutput;
}

trait TestCmd {
    fn test(&mut self, config: &Config, name: Option<String>) -> BuildkOutput;
}

trait TreeCmd {
    fn tree(&mut self, config: &Config) -> BuildkOutput;
}

impl Commands {
    pub fn apply(&mut self, config: &Config) -> BuildkOutput {

        match self {
            Commands::Build { ref set } => self.build(config, set.clone()),
            Commands::Clean => self.clean(config),
            Commands::Config => self.config(config),
            Commands::Deps => self.deps(config),
            Commands::Fetch => self.fetch(config),
            Commands::Release => self.release(config),
            Commands::Tree => self.tree(config),
            Commands::Run { ref name } => self.run(config, name.clone()),
            Commands::Test { ref name } => self.test(config, name.clone()),
        }
    }

    fn load_cache(&self, config: &Config) -> Cache {
        let kotlin_home = get_kotlin_home();
        let cache_dir = &config.manifest.project.out.cache;

        Cache::load(&kotlin_home, cache_dir)
    }

    fn execute(
        &self,
        output: &mut BuildkOutput,
        config: &Config,
        cmd: &ProcessBuilder,
        extra_fingerprint: u64,
    ) -> BuildkOutput {
        let mut cache = self.load_cache(config);
        match cache.cache_command(cmd, extra_fingerprint) {
            Ok(cache_res) => {
                output
                    .conclude(cache_res.conclusion)
                    .stdout(cache_res.stdout.unwrap_or("".to_owned()))
                    .status(cache_res.status);

                if let Some(stderr) = cache_res.stderr {
                    output
                        .conclude(PartialConclusion::FAILED)
                        .stderr(stderr);
                }

                output.to_owned()
            }

            Err(err) => {
                let err = err.to_string().as_red();

                println!("\r{err:#}");

                output
                    .conclude(PartialConclusion::FAILED)
                    .stderr(err.to_string())
                    .to_owned()
            },
        }
    }
}

pub struct Command {
    pub version: String,
    //test_libs: Vec<PathBuf>,
    pub client: Client,
}

impl Command {
    // TODO: This should be loaded as an init to see if kotlin is installed
    pub fn new(config: &Config) -> Result<Command> {
        let kotlin_home = get_kotlin_home();
        let mut cache = Cache::load(&kotlin_home, &config.manifest.project.out.cache);

        let mut kotlinc = Command {
            version: "unknown".to_string(),
            /*test_libs: vec![
                kotlin_home.join("libexec/lib/kotlin-test-junit5.jar"),
                kotlin_home.join("libexec/lib/kotlin-test.jar"),
            ],*/
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
