use std::env::args;
use config::config::Config;

use config::LazyConfig;
use kotlin::Kotlin;
use util::buildk_output::BuildkOutput;
use util::cmd::Cmd;
use util::Conclusion;
use util::timer::Timer;

fn main() {
    let timer = Timer::start();
    let mut lazy_config = LazyConfig::new();
    let config = lazy_config.get_mut();

    // println!("{config}");

    let commands = args().skip(1).flat_map(|arg| Cmd::from(arg)).collect::<Vec<Cmd>>();
    let mut kotlinc = Kotlin::new(&config).expect("kotlin expected");

    let outputs = commands.iter().map(|cmd| {
        let output = execute(cmd, &config, &mut kotlinc);
        println!("{:<6} {:<12} ▸ {}", output.conclusion(), cmd, output.elapsed());
        output
    }).collect::<Vec<BuildkOutput>>();

    let errors = outputs.iter()
        .filter(|output| output.get_stderr().is_some())
        .map(|output| output.get_stderr().unwrap())
        .fold(String::new(), |errors, output| {
            format!("{errors}\n{output}")
        });

    println!();

    if errors.is_empty() {
        println!("{} in {}", Conclusion::SUCCESS, timer.elapsed());
    } else {
        println!("{} in {}", Conclusion::FAILED, timer.elapsed());
        println!("{errors}");
    }

    fn execute(command: &Cmd, config: &Config, kotlinc: &mut Kotlin) -> BuildkOutput {
        match command {
            Cmd::Clean => kotlinc.clean(config),
            Cmd::BuildSrc => kotlinc.build_src(config),
            Cmd::BuildTest => kotlinc.build_test(config),
            Cmd::Test => kotlinc.run_tests(config),
            Cmd::Run => kotlinc.run(config),
            Cmd::Release => kotlinc.target(config),
        }
    }
}
