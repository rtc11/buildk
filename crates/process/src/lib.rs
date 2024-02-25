use core::fmt;
use std::{
    collections::BTreeMap,
    env,
    ffi::{OsStr, OsString},
    fmt::Formatter,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Output, Stdio},
};

use anyhow::{Context, Result};

use manifest::config::Config;

pub mod java;
pub mod kotlin;

pub trait Process<'a> {
    type Item;

    fn new(config: &'a Config) -> Result<Self::Item>;
}

#[derive(Clone, Debug)]
pub struct ProcessBuilder {
    program: OsString,
    args: Vec<OsString>,
    env: BTreeMap<String, Option<OsString>>,
    cwd: Option<OsString>,
    stdin: Option<Vec<u8>>,
}

impl fmt::Display for ProcessBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "`")?;

        for (key, val) in self.env.iter() {
            if let Some(val) = val {
                let val = val.to_string_lossy();
                write!(f, "{key}={val}")?;
            }
        }

        write!(f, "{}", self.get_program().to_string_lossy())?;

        for arg in self.get_args() {
            write!(f, " {}", arg.to_string_lossy())?;
        }

        write!(f, "`")
    }
}

#[allow(dead_code)]
impl ProcessBuilder {
    pub fn new<T: AsRef<OsStr>>(cmd: T) -> ProcessBuilder {
        ProcessBuilder {
            program: cmd.as_ref().to_os_string(),
            args: Vec::new(),
            cwd: None,
            env: BTreeMap::new(),
            stdin: None,
        }
    }

    pub fn program<T: AsRef<OsStr>>(&mut self, cmd: T) -> &mut ProcessBuilder {
        self.program = cmd.as_ref().to_os_string();
        self
    }

    pub fn arg<T: AsRef<OsStr>>(&mut self, arg: T) -> &mut ProcessBuilder {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    pub fn classpath<T: AsRef<OsStr>>(&mut self, path: T) -> &mut ProcessBuilder {
        self.args(&[OsString::from("-cp"), path.as_ref().to_os_string()])
    }

    pub fn jar<T: AsRef<OsStr>>(&mut self, jar_path: T) -> &mut ProcessBuilder {
        self.args(&[OsString::from("-jar"), jar_path.as_ref().to_os_string()])
    }

    pub fn classpaths(&mut self, paths: Vec<&PathBuf>) -> &mut ProcessBuilder {
        let classpath = paths
            .into_iter()
            .map(|path| path.as_os_str())
            .collect::<Vec<&OsStr>>()
            .as_slice()
            .join(&OsString::from(":"));

        self.args(&[OsString::from("-cp"), classpath])
    }

    pub fn sources<T: AsRef<OsStr>>(&mut self, path: T) -> &mut ProcessBuilder {
        self.args.push(path.as_ref().to_os_string());
        self
    }

    pub fn include_runtime(&mut self) -> &mut ProcessBuilder {
        self.args.push(OsString::from("-include-runtime"));
        self
    }

    pub fn destination<T: AsRef<OsStr>>(&mut self, destination: T) -> &mut ProcessBuilder {
        self.args(&[OsString::from("-d"), destination.as_ref().to_os_string()])
    }

    pub fn test_report<T: AsRef<OsStr>>(&mut self, report_dir: T) -> &mut ProcessBuilder {
        self.args(&[
            OsString::from("--reports-dir"),
            report_dir.as_ref().to_os_string(),
        ])
    }

    pub fn args<T: AsRef<OsStr>>(&mut self, args: &[T]) -> &mut ProcessBuilder {
        self.args
            .extend(args.iter().map(|a| a.as_ref().to_os_string()));
        self
    }

    pub fn cwd<T: AsRef<OsStr>>(&mut self, path: T) -> &mut ProcessBuilder {
        self.cwd = Some(path.as_ref().to_os_string());
        self
    }

    pub fn env<T: AsRef<OsStr>>(&mut self, key: &str, val: T) -> &mut ProcessBuilder {
        self.env
            .insert(key.to_string(), Some(val.as_ref().to_os_string()));
        self
    }

    pub fn get_program(&self) -> &OsString {
        &self.program
    }
    pub fn get_args(&self) -> impl Iterator<Item=&OsString> {
        self.args.iter()
    }
    pub fn get_cwd(&self) -> Option<&Path> {
        self.cwd.as_ref().map(Path::new)
    }
    pub fn get_envs(&self) -> &BTreeMap<String, Option<OsString>> {
        &self.env
    }

    pub fn get_env(&self, var: &str) -> Option<OsString> {
        self.env
            .get(var)
            .cloned()
            .or_else(|| Some(env::var_os(var)))
            .and_then(|s| s) // flatmap
    }

    pub fn build_command(&self) -> Command {
        let mut cmd = self.build_command_without_args();
        for arg in &self.args {
            cmd.arg(arg);
        }
        cmd
    }

    fn build_command_without_args(&self) -> Command {
        let mut command = Command::new(&self.program);
        if let Some(cwd) = self.get_cwd() {
            command.current_dir(cwd);
        }
        for (k, v) in &self.env {
            match *v {
                Some(ref v) => command.env(k, v),
                None => command.env_remove(k),
            };
        }
        command
    }

    pub fn output(&self) -> Result<Output> {
        self._output()
            .with_context(|| ProcessError::could_not_execute(self))
    }

    fn _output(&self) -> io::Result<Output> {
        let mut cmd = self.build_command();
        let mut child = piped(&mut cmd, self.stdin.is_some()).spawn()?;
        if let Some(stdin) = &self.stdin {
            child.stdin.take().unwrap().write_all(stdin)?;
        }
        child.wait_with_output()
    }
}

fn piped(cmd: &mut Command, pipe_stdin: bool) -> &mut Command {
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(if pipe_stdin {
            Stdio::piped()
        } else {
            Stdio::null()
        })
}

#[derive(Debug)]
pub struct ProcessError {
    pub desc: String,
    pub code: Option<i32>,
    pub stdout: Option<Vec<u8>>,
    pub stderr: Option<Vec<u8>>,
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { self.desc.fmt(f) }
}

impl std::error::Error for ProcessError {}

impl ProcessError {
    pub fn new(msg: &str, status: Option<ExitStatus>, output: Option<&Output>) -> ProcessError {
        let exit = match status {
            Some(s) => exit_status_to_string(s),
            None => "never executed".to_string()
        };

        Self::new_with_raw_output(
            msg,
            status.and_then(|s| s.code()),
            &exit,
            output.map(|output| output.stdout.as_slice()),
            output.map(|output| output.stderr.as_slice()),
        )
    }

    pub fn new_with_raw_output(
        msg: &str,
        code: Option<i32>,
        status: &str,
        stdout: Option<&[u8]>,
        stderr: Option<&[u8]>,
    ) -> ProcessError {
        let mut desc = format!("{} ({})", msg, status);

        if let Some(out) = stdout {
            match std::str::from_utf8(out) {
                Ok(s) if !s.trim().is_empty() => {
                    desc.push_str("\n--- stdout\n");
                    desc.push_str(s);
                }
                Ok(..) | Err(..) => {}
            }
        }

        if let Some(out) = stderr {
            match std::str::from_utf8(out) {
                Ok(s) if !s.trim().is_empty() => {
                    desc.push_str("\n--- stderr\n");
                    desc.push_str(s);
                }
                Ok(..) | Err(..) => {}
            }
        }

        ProcessError {
            desc,
            code,
            stdout: stdout.map(|s| s.to_vec()),
            stderr: stderr.map(|s| s.to_vec()),
        }
    }

    pub fn could_not_execute(cmd: impl fmt::Display) -> ProcessError {
        ProcessError::new(&format!("could not execute process {cmd}"), None, None)
    }
}

#[cfg(unix)]
pub fn exit_status_to_string(status: ExitStatus) -> String {
    use std::os::unix::process::*;

    if let Some(signal) = status.signal() {
        let name = match signal as libc::c_int {
            libc::SIGABRT => ", SIGABRT: process abort signal",
            libc::SIGALRM => ", SIGALRM: alarm clock",
            libc::SIGFPE => ", SIGFPE: erroneous arithmetic operation",
            libc::SIGHUP => ", SIGHUP: hangup",
            libc::SIGILL => ", SIGILL: illegal instruction",
            libc::SIGINT => ", SIGINT: terminal interrupt signal",
            libc::SIGKILL => ", SIGKILL: kill",
            libc::SIGPIPE => ", SIGPIPE: write on a pipe with no one to read",
            libc::SIGQUIT => ", SIGQUIT: terminal quit signal",
            libc::SIGSEGV => ", SIGSEGV: invalid memory reference",
            libc::SIGTERM => ", SIGTERM: termination signal",
            libc::SIGBUS => ", SIGBUS: access to undefined memory",
            libc::SIGSYS => ", SIGSYS: bad system call",
            libc::SIGTRAP => ", SIGTRAP: trace/breakpoint trap",
            _ => "",
        };
        format!("signal: {}{}", signal, name)
    } else {
        status.to_string()
    }
}

fn try_from(
    cmd: &ProcessBuilder,
    output: std::process::Output,
) -> Result<cache::output::Output> {
    let mut action = String::from(cmd.get_program().to_string_lossy());
    for arg in cmd.get_args() {
        action.push_str(&format!(" {}", arg.to_string_lossy()));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("{}: {:?}", e, e.as_bytes()))
        .with_context(|| "Failed to convert stdout to utf8")?;

    let stderr = String::from_utf8(output.stderr)
        .map_err(|e| anyhow::anyhow!("{}: {:?}", e, e.as_bytes()))
        .with_context(|| "Failed to convert stderr to utf8")?;

    let status = if output.status.success() { String::new() } else { exit_status_to_string(output.status) };

    Ok(cache::output::Output {
        action: action.to_owned(),
        success: output.status.success(),
        status,
        code: output.status.code(),
        stdout,
        stderr,
    })
}

