use core::fmt;
use std::{env, io};
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fmt::Formatter;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};

use anyhow::{Context, Result};

use crate::process_error::ProcessError;

#[derive(Clone, Debug)]
pub struct ProcessBuilder {
    program: OsString,
    args: Vec<OsString>,
    env: BTreeMap<String, Option<OsString>>,
    /// Current working directory (to run the program in)
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

    pub fn program<T: AsRef<OsStr>>(&mut self, program: T) -> &mut ProcessBuilder {
        self.program = program.as_ref().to_os_string();
        self
    }

    pub fn arg<T: AsRef<OsStr>>(&mut self, arg: T) -> &mut ProcessBuilder {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    pub fn classpath<T: AsRef<OsStr>>(&mut self, path: T) -> &mut ProcessBuilder {
        self.args(&[OsString::from("-cp"), path.as_ref().to_os_string()])
    }

    pub fn classpaths(&mut self, paths: Vec<PathBuf>) -> &mut ProcessBuilder {
        let classpath = paths.iter()
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
        self.args(&[OsString::from("--reports-dir"), report_dir.as_ref().to_os_string()])
    }

    pub fn args<T: AsRef<OsStr>>(&mut self, args: &[T]) -> &mut ProcessBuilder {
        self.args.extend(args.iter().map(|a| a.as_ref().to_os_string()));
        self
    }

    pub fn cwd<T: AsRef<OsStr>>(&mut self, path: T) -> &mut ProcessBuilder {
        self.cwd = Some(path.as_ref().to_os_string());
        self
    }

    pub fn env<T: AsRef<OsStr>>(&mut self, key: &str, val: T) -> &mut ProcessBuilder {
        self.env.insert(key.to_string(), Some(val.as_ref().to_os_string()));
        self
    }

    pub fn get_program(&self) -> &OsString { &self.program }
    pub fn get_args(&self) -> impl Iterator<Item=&OsString> { self.args.iter() }
    pub fn get_cwd(&self) -> Option<&Path> { self.cwd.as_ref().map(Path::new) }
    pub fn get_envs(&self) -> &BTreeMap<String, Option<OsString>> { &self.env }

    pub fn get_env(&self, var: &str) -> Option<OsString> {
        self.env.get(var).cloned()
            .or_else(|| Some(env::var_os(var)))
            .and_then(|s| s) // flatmap
    }

    pub fn stdin<T: Into<Vec<u8>>>(&mut self, stdin: T) -> &mut Self {
        self.stdin = Some(stdin.into());
        self
    }

    pub fn status(&self) -> Result<ExitStatus> {
        self.build_command().spawn()?.wait().with_context(|| ProcessError::could_not_execute(self))
    }

    pub fn exec(&self) -> Result<()> {
        let exit = self.status()?;
        if exit.success() {
            Ok(())
        } else {
            let error = ProcessError::new(
                &format!("process dit not exit successfully: {}", self),
                Some(exit),
                None,
            );
            Err(error.into())
        }
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
        self._output().with_context(|| ProcessError::could_not_execute(self))
    }

    fn _output(&self) -> io::Result<Output> {
        let mut cmd = self.build_command();
        let mut child = piped(&mut cmd, self.stdin.is_some()).spawn()?;
        if let Some(stdin) = &self.stdin {
            child.stdin.take().unwrap().write_all(stdin)?;
        }
        child.wait_with_output()
    }

    pub fn exec_with_output(&self) -> Result<Output> {
        let output = self.output()?;
        if output.status.success() {
            Ok(output)
        } else {
            let error = ProcessError::new(
                &format!("process didn't exit successfully: {}", self),
                Some(output.status),
                Some(&output),
            );
            Err(error.into())
        }
    }
}

fn piped(cmd: &mut Command, pipe_stdin: bool) -> &mut Command {
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(if pipe_stdin { Stdio::piped() } else { Stdio::null() })
}
