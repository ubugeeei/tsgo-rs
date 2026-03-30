use crate::{
    Result,
    fast::{CompactString, SmallVec},
};
use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Stdio},
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

/// Immutable process template for launching `typescript-go`.
///
/// The command is cheap to clone and can be reused across multiple clients.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use tsgo_rs_core::TsgoCommand;
///
/// let command = TsgoCommand::new("/opt/bin/tsgo")
///     .with_cwd("/workspace")
///     .with_env("TSGO_TRACE", "0");
///
/// assert_eq!(command.cwd(), &PathBuf::from("/workspace"));
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TsgoCommand {
    executable: PathBuf,
    cwd: PathBuf,
    env: SmallVec<[(CompactString, CompactString); 4]>,
}

impl TsgoCommand {
    /// Creates a new command template rooted at the current working directory.
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            env: SmallVec::new(),
        }
    }

    /// Returns a clone with a different working directory.
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = cwd.into();
        self
    }

    /// Returns a clone with an extra environment variable.
    pub fn with_env(
        mut self,
        key: impl Into<CompactString>,
        value: impl Into<CompactString>,
    ) -> Self {
        let key = key.into();
        let value = value.into();
        if let Some((_, entry)) = self.env.iter_mut().find(|(existing, _)| existing == key) {
            *entry = value;
        } else {
            self.env.push((key, value));
        }
        self
    }

    /// Returns the working directory used for child processes.
    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    /// Spawns a child process with piped stdin/stdout for request/response flows.
    pub fn spawn_async<I, S>(&self, args: I) -> std::io::Result<Child>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.spawn(args)
    }

    /// Spawns a child process for blocking protocols such as the msgpack transport.
    pub fn spawn_blocking<I, S>(&self, args: I) -> std::io::Result<std::process::Child>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.spawn(args)
    }

    fn spawn<I, S>(&self, args: I) -> std::io::Result<std::process::Child>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = std::process::Command::new(&self.executable);
        command
            .args(args)
            .current_dir(&self.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .envs(
                self.env
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str())),
            );
        command.spawn()
    }
}

/// Owns a child process and guarantees it is eventually terminated.
#[derive(Debug)]
pub struct AsyncChildGuard {
    child: Mutex<Option<Child>>,
}

impl AsyncChildGuard {
    /// Wraps a running child process.
    pub fn new(child: Child) -> Self {
        Self {
            child: Mutex::new(Some(child)),
        }
    }

    /// Waits for graceful exit and force-kills the process when the deadline expires.
    pub async fn shutdown(&self, wait_for: Duration) -> Result<()> {
        let mut child = self.child.lock().unwrap();
        let Some(mut child) = child.take() else {
            return Ok(());
        };
        let deadline = Instant::now() + wait_for;
        loop {
            if child.try_wait()?.is_some() {
                return Ok(());
            }
            if Instant::now() >= deadline {
                child.kill()?;
                child.wait()?;
                return Ok(());
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for AsyncChildGuard {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.try_lock()
            && let Some(child) = child.as_mut()
        {
            let _ = child.kill();
        }
    }
}
