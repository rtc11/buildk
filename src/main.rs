use anyhow::Result;
use itertools::Itertools;
use util::{
    buildk_output::BuildkOutput,
    terminal::{Printable, Terminal},
};

use command::Command;
use manifest::config::Config;

fn main() -> Result<()> {
    //let timer = Timer::start();
    let config = Config::default();
    let mut command = Command::new(&config).expect("kotlin expected");
    let mut terminal = Terminal::default();

    let executions = std::env::args()
        .skip(1)
        .flat_map(command::Option::from)
        .unique()
        .map(|option| execute(&option, &config, &mut command, &mut terminal))
        .collect::<Vec<_>>();

    let errors = executions
        .iter()
        .filter(|output| output.get_status() != 0)
        .filter_map(|output| output.get_stderr())
        .fold(String::new(), |errors, output| {
            format!("{errors}\n{output}")
        });

    let outputs = executions
        .iter()
        .filter(|output| output.get_status() == 0)
        .filter_map(|output| output.get_stdout())
        .fold(String::new(), |outputs, output| {
            format!("{outputs}\n{output}")
        });

    if !errors.is_empty() {
        terminal.print(&errors);
        config.print(&mut terminal);
        //println!("{} in {}", Conclusion::FAILED, timer.elapsed());
        //println!("{errors}");
        //println!("{config}\n");
    } else {
        //println!("{} in {}", Conclusion::SUCCESS, timer.elapsed());
    }

    if !outputs.is_empty() {
        terminal.print(&outputs);
    }

    Ok(())
}

fn execute(
    option: &command::Option,
    config: &Config,
    command: &mut Command,
    //command_opt: Option<String>,
    terminal: &mut Terminal,
) -> BuildkOutput {
    terminal.start_spin(0, &option.to_string());

    let output = match option {
        command::Option::Clean => command.clean(config, terminal),
        command::Option::Fetch => command.fetch_async(config, terminal),
        command::Option::BuildSrc => command.build_src(config, terminal),
        command::Option::BuildTest => command.build_test(config, terminal),
        command::Option::Test => command.run_tests(config, terminal),
        command::Option::Release => command.release(config, terminal),
        command::Option::Run => command.run(config, terminal),
        command::Option::Deps => command.deps(config, terminal),
        command::Option::BuildTree => command.build_tree(config, terminal),
        command::Option::Config => command.config(config, terminal),
        command::Option::Help => command.help(config, terminal),
    };

    terminal.stop_spin(0);


    terminal.print_row(0, &format!(
        "\r{:<6} {:<12} ▸ {}",
        output.conclusion().color_symbol(),
        option.to_string(),
        output.elapsed()
    ));

    output
}
