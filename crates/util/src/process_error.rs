use core::fmt;
use std::fmt::Formatter;
use std::process::{ExitStatus, Output};

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
