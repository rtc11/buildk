use anyhow::Result;
use util::terminal::Terminal;
use command::{Command, Cli};
use manifest::config::Config;

fn main() -> Result<()> {
    let config = Config::default();
    let _ = Command::new(&config).expect("kotlin expected");
    let mut terminal = Terminal::default();


    //terminal.start_spin(0, &option.to_string());
    let mut commands = Cli::commands();
    let output = commands.apply(&config); 

    //terminal.stop_spin(0);

    terminal.print_row(0, &format!(
        "\r{:<6} {:<12} ▸ {}",
        output.conclusion().color_symbol(),
        output.get_command(),
        output.elapsed()
    ));

    if let Some(stdout) = output.get_stdout() {
        println!("\r\n{stdout}");
    }

    Ok(())
}

