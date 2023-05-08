use crate::PartialConclusion;
use crate::timer::Timer;

#[derive(Clone)]
pub struct BuildkOutput {
    status: PartialConclusion,
    stdout: Option<String>,
    stderr: Option<String>,
    cache_hit: bool,
    timed: Timer,
}

impl Default for BuildkOutput {
    fn default() -> Self {
        BuildkOutput {
            status: PartialConclusion::INIT,
            stdout: None,
            stderr: None,
            cache_hit: false,
            timed: Timer::start(),
        }
    }
}

impl BuildkOutput {
    pub fn conclude(&mut self, status: PartialConclusion) -> &mut Self {
        if self.status == PartialConclusion::INIT {
            self.status = status;
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
    pub fn cache_miss(&mut self) -> &mut Self {
        self.cache_hit = false;
        self
    }
    pub fn elapsed(&self) -> String {
        self.timed.elapsed()
    }
    pub fn conclusion(&self) -> PartialConclusion {
        self.status.clone()
    }
    pub fn get_stderr(&self) -> Option<String> {
        self.stderr.clone()
    }
    pub fn get_stdout(&self) -> Option<String> {
        self.stdout.clone()
    }
}
