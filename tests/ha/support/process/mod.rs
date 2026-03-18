use std::{ffi::OsStr, path::Path, process::Command};

use crate::support::error::{HarnessError, Result};

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

pub fn run<I, S, E, K, V>(
    executable: &Path,
    context: impl Into<String>,
    args: I,
    env: E,
) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    run_with_options(executable, None, context.into(), args, env)
}

pub fn run_in_dir<I, S, E, K, V>(
    executable: &Path,
    cwd: &Path,
    context: impl Into<String>,
    args: I,
    env: E,
) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    run_with_options(executable, Some(cwd), context.into(), args, env)
}

fn run_with_options<I, S, E, K, V>(
    executable: &Path,
    cwd: Option<&Path>,
    context: String,
    args: I,
    env: E,
) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    ensure_absolute_executable(executable)?;

    let mut command = Command::new(executable);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    command.env_clear();
    command.envs(env);

    let output = command.output().map_err(|source| HarnessError::Io {
        path: executable.to_path_buf(),
        source,
    })?;
    if output.status.success() {
        return String::from_utf8(output.stdout).map_err(|source| HarnessError::Utf8 {
            context: format!("decoding stdout for {context}"),
            source,
        });
    }

    Err(HarnessError::CommandFailed {
        executable: executable.to_path_buf(),
        context,
        status: render_exit_status(output.status.code()),
        stdout: String::from_utf8_lossy(output.stdout.as_slice()).into_owned(),
        stderr: String::from_utf8_lossy(output.stderr.as_slice()).into_owned(),
    })
}

fn render_exit_status(code: Option<i32>) -> String {
    code.map(|value| value.to_string())
        .unwrap_or_else(|| "terminated by signal".to_string())
}
