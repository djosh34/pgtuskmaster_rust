use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process::Command,
};

use crate::support::error::{HarnessError, Result};

#[derive(Clone, Debug, Default)]
pub struct CommandSpec {
    pub executable: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
    pub context: String,
}

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: Vec<u8>,
}

impl CommandOutput {
    pub fn stdout_text(&self, context: impl Into<String>) -> Result<String> {
        String::from_utf8(self.stdout.clone()).map_err(|source| HarnessError::Utf8 {
            context: context.into(),
            source,
        })
    }

}

impl CommandSpec {
    pub fn new(
        executable: impl Into<PathBuf>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            executable: executable.into(),
            args: Vec::new(),
            cwd: None,
            env: BTreeMap::new(),
            context: context.into(),
        }
    }

    pub fn args<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args.extend(values.into_iter().map(|value| value.as_ref().to_os_string()));
        self
    }

    pub fn cwd(mut self, value: impl Into<PathBuf>) -> Self {
        self.cwd = Some(value.into());
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

pub fn ensure_absolute_path(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err(HarnessError::message(format!(
            "expected an absolute executable path, got `{}`",
            path.display()
        )));
    }
    Ok(())
}

pub fn ensure_absolute_executable(path: &Path) -> Result<()> {
    ensure_absolute_path(path)?;
    if !path.exists() {
        return Err(HarnessError::message(format!(
            "required executable does not exist: `{}`",
            path.display()
        )));
    }
    if !path.is_file() {
        return Err(HarnessError::message(format!(
            "required executable is not a file: `{}`",
            path.display()
        )));
    }
    Ok(())
}

pub fn run(spec: CommandSpec) -> Result<CommandOutput> {
    ensure_absolute_executable(spec.executable.as_path())?;

    let mut command = Command::new(spec.executable.as_path());
    command.args(spec.args.iter());
    if let Some(cwd) = spec.cwd.as_ref() {
        command.current_dir(cwd);
    }
    command.env_clear();
    command.envs(spec.env.iter());

    let output = command.output().map_err(|source| HarnessError::Io {
        path: spec.executable.clone(),
        source,
    })?;
    if output.status.success() {
        return Ok(CommandOutput {
            stdout: output.stdout,
        });
    }

    Err(HarnessError::CommandFailed {
        executable: spec.executable,
        context: spec.context,
        status: render_exit_status(output.status.code()),
        stdout: String::from_utf8_lossy(output.stdout.as_slice()).into_owned(),
        stderr: String::from_utf8_lossy(output.stderr.as_slice()).into_owned(),
    })
}

fn render_exit_status(code: Option<i32>) -> String {
    code.map(|value| value.to_string())
        .unwrap_or_else(|| "terminated by signal".to_string())
}
