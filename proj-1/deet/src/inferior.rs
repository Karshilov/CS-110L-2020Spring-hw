use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::os::unix::process::CommandExt;
use std::process::Child;
use std::process::Command;

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

pub struct Inferior {
    child: Child,
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>) -> Option<Inferior> {
        unsafe {
            let mut cmd: Command = Command::new(target);
            cmd.args(args).pre_exec(|| { child_traceme() });
            let ret = Inferior { child: cmd.spawn().ok()? };
            let status = ret.wait(None).ok()?;
            match status {
                Status::Stopped(sign, pts) => {
                    if sign == signal::Signal::SIGTRAP { return Some(ret); }
                    else {
                        println!("The stop-signal is {} and it has executed at {}", sign, pts);
                        return None;
                    }
                }
                Status::Exited(_) => { return None; }
                Status::Signaled(_) => { return None; }
            }
        }
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }

    pub fn cont_process(&self) -> Result<Status, nix::Error> {
        match ptrace::cont(self.pid(), None) {
            Ok(_) => { return self.wait(None); }
            Err(err) => Err(err)
        }
    }

    pub fn kill(&mut self) -> Result<std::process::ExitStatus, std::io::Error> {
        match self.child.kill() {
            Ok(_) => {
                let status = self.child.wait()?;
                return Ok(status);
            },
            Err(err) => Err(err),
        }
    }

    pub fn print_backtrace(&self) -> Result<(), nix::Error> {
        let reg = ptrace::getregs(self.pid())?;
        println!("%rip register : {:#x}", reg.rip);
        Ok(())
    }
}
