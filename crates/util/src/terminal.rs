//use multi_spinner::Spinner;
use spinners::{Spinner, Spinners};
use std::collections::HashMap;

#[derive(Default)]
pub struct Terminal {
    spinners: HashMap<u16, Spinner>,
}

pub trait Printable {
    fn print(&self, terminal: &mut Terminal);
}


impl Terminal {
    pub fn print(&mut self, text: &str) {
        println!("\r{text}");
    }

    pub fn print_row(&mut self, _row: u16, text: &str) {
        println!("\r{text}");
    }

    pub fn start_spin(&mut self, row: u16, text: &str) {
        let spinner = Spinner::new(Spinners::Dots7, text.into());
        self.spinners.insert(row, spinner);
    }

    pub fn stop_spin(&mut self, row: u16) {
        if let Some(mut spinner) = self.spinners.remove(&row) {
            spinner.stop();
        }
    }
    /*
    pub fn next_row(&self) -> u16 {
        match self.rows.iter().max() {
            Some(row) => row + 1,
            None => 0,
        }
    }

    pub fn clear_row(&mut self, row: u16) {
        if let Ok(mut stdout) = self.stdout.lock() {
            queue!(stdout, MoveToRow(row), Clear(ClearType::CurrentLine))
                .expect("Failed to clear row");
        }
    }
    pub fn remove_row(&mut self, row: u16) {
        self.clear_row(row);
        self.rows.remove(&row);
    }

    pub fn print(&mut self, text: &str) {
        let next_row = self.next_row();
        self.print_row(next_row, text);
    }

    pub fn print_row(&mut self, row: u16, text: &str) {
        if !self.rows.contains(&row) {
            self.rows.insert(row);
        }

        if let Ok(mut stdout) = self.stdout.lock() {

            queue!(
                stdout,
                MoveToRow(row),
                Print(format!("{text}\r")),
            )
            .expect("Failed to print");
        }
    }

    pub fn start_spin(&mut self, row: u16, text: &str) {
        if !self.rows.contains(&row) {
            self.rows.insert(row);
        }

        let spinner = Spinner::builder()
            .stdout(self.stdout.clone())
            .msg(text.into())
            .row(row.into())
            .start();

        self.spinners.insert(row, spinner);
    }

    pub fn stop_spin(&mut self, row: u16) {
        if let Some(mut spinner) = self.spinners.remove(&row) {
            spinner.stop().expect("Failed to stop spinner");
            self.remove_row(row);
        }
    }
    */
}
