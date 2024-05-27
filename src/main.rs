use anyhow::Result;
use command::Cli;
use manifest::config::BuildK;
use util::terminal::Terminal;

fn main() -> Result<()> {
    let buildk = BuildK::new();
    let mut terminal = Terminal::default();
    let mut cli = Cli::init();
    let output = cli.command.apply(&buildk);

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
