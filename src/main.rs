use anyhow::Result;
use command::Cli;
use manifest::config::Config;
use util::terminal::Terminal;

fn main() -> Result<()> {
    let config = Config::new();
    let mut terminal = Terminal::default();

    //terminal.start_spin(0, &option.to_string());
    let mut cli = Cli::init();
    let output = cli.command.apply(&config);

    //terminal.stop_spin(0);

    if !cli.is_quiet() {
        terminal.print_row(
            0,
            &format!(
                "\r{:<6} {:<12} ▸ {}",
                output.conclusion().color_symbol(),
                output.get_command(),
                output.elapsed()
            ),
        );
        if let Some(stdout) = output.get_stdout() {
            println!("\r\n{stdout}");
        }
    }

    Ok(())
}
