use std::env::args;
use config::config::Config;

use kotlin::Kotlin;
use util::buildk_output::BuildkOutput;
use util::cmd::Cmd;
use util::Conclusion;
use util::timer::Timer;

fn main() {
    option_env!("PATH").unwrap();
    let timer = Timer::start();
    let config = Config::default();
    let mut kotlin = Kotlin::new(&config).expect("kotlin expected");
    let errors = args()
        .skip(1)
        .flat_map(Cmd::from)
        .map(|cmd| execute(&cmd, &config, &mut kotlin))
        .filter_map(|output| output.get_stderr())
        .fold(String::new(), |errors, output| format!("{errors}\n{output}"));

    if errors.is_empty() {
        println!("{} in {}", Conclusion::SUCCESS, timer.elapsed());
    } else {
        println!("{} in {}", Conclusion::FAILED, timer.elapsed());
        println!("{errors}");
        println!("{config}");
    }

    fn execute(command: &Cmd, config: &Config, kotlinc: &mut Kotlin) -> BuildkOutput {
        let output = match command {
            Cmd::Clean => kotlinc.clean(config),
            Cmd::BuildSrc => kotlinc.build_src(config),
            Cmd::BuildTest => kotlinc.build_test(config),
            Cmd::Test => kotlinc.run_tests(config),
            Cmd::Release => kotlinc.release(config),
            Cmd::Run => {
                let run = kotlinc.run(config);
                if let Some(stdout) = run.get_stdout() {
                    println!("{stdout}");
                }
                run
            },
        };
        println!("{:<6} {:<12} ▸ {}", output.conclusion(), command, output.elapsed());
        output
    }
}
