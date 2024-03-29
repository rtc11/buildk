use crate::PartialConclusion;
use crate::timer::Timer;

#[derive(Clone, Debug)]
pub struct BuildkOutput {
    command: String, 
    conclusion: PartialConclusion,
    stdout: Option<String>,
    stderr: Option<String>,
    status: i32,
    cache_hit: bool,
    timed: Timer,
}

impl Default for BuildkOutput {
    fn default() -> Self {
        BuildkOutput {
            command: String::new(),
            conclusion: PartialConclusion::INIT,
            stdout: None,
            stderr: None,
            status: 0,
            cache_hit: false,
            timed: Timer::start(),
        }
    }
}
pub trait WithBKOutput {
    fn add_to_output<'a>(&'a self, out: &'a mut BuildkOutput) -> &'a mut BuildkOutput;
}

impl BuildkOutput {
    pub fn new(command: &str) -> Self {
        BuildkOutput {
            command: command.to_string(),
            ..Default::default()
        }
    }

    pub fn apply(&mut self, other: BuildkOutput) -> Self {
        if other.conclusion != PartialConclusion::INIT {
            self.conclusion = other.conclusion;
        }
        if other.stdout.is_some() {
            self.stdout = other.stdout;
        }
        if other.stderr.is_some() {
            self.stderr = other.stderr;
        }
        if other.status != 0 {
            self.status = other.status;
        }
        self.to_owned()
    }
    
    pub fn get_command(&self) -> &str {
        &self.command
    }

    pub fn conclude(&mut self, status: PartialConclusion) -> &mut Self {
        if self.conclusion == PartialConclusion::INIT {
            self.conclusion = status;
        }
        self
    }
    pub fn cache_hit(&mut self) -> &mut Self {
        self.cache_hit = true;
        self
    }
    pub fn stdout(&mut self, stdout: String) -> &mut Self {
        if !stdout.is_empty() {
            self.stdout = Some(stdout);
        }
        self
    }
    pub fn stderr(&mut self, stderr: String) -> &mut Self {
        if !stderr.is_empty() {
            self.stderr = Some(stderr);
        }
        self
    }

    pub fn append_stderr(&mut self, stderr: String) -> &mut Self {
        if let Some(err) = &self.stderr {
            self.stderr = Some(err.to_owned() + "\n" + &stderr);
            self
        } else {
            self.stderr(stderr)
        }
    }

    pub fn status(&mut self, status: i32) -> &mut Self {
        self.status = status;
        self
    }
    pub fn cache_miss(&mut self) -> &mut Self {
        self.cache_hit = false;
        self
    }
    pub fn get_status(&self) -> i32 {
        self.status
    }
    pub fn elapsed(&self) -> String {
        self.timed.elapsed()
    }
    pub fn conclusion(&self) -> PartialConclusion {
        self.conclusion.clone()
    }
    pub fn get_stderr(&self) -> Option<String> {
        self.stderr.clone()
    }
    pub fn get_stdout(&self) -> Option<String> {
        self.stdout.clone()
    }
}
