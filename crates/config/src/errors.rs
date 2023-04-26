#[derive(Debug)]
pub struct CliError {
    pub error: Option<anyhow::Error>,
    pub exit_code: i32,
}

impl CliError {
    pub fn new(error: anyhow::Error, exit_code: i32) -> CliError {
        CliError {
            error: Some(error),
            exit_code,
        }
    }

    pub fn code(exit_code: i32) -> CliError {
        CliError {
            error: None,
            exit_code,
        }
    }
}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> CliError {
        CliError::new(err, 101)
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError::new(err.into(), 1)
    }
}
