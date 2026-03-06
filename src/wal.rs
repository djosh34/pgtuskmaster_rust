use std::path::{Path, PathBuf};

/// A rendered command (program + argv) intended to be executed without a shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum WalHelperError {
    #[error("{0}")]
    Message(String),
}

pub fn render_archive_push(
    pgdata: &Path,
    wal_path: &str,
) -> Result<RenderedCommand, WalHelperError> {
    let rendered =
        crate::backup::archive_command::render_archive_push_from_pgdata(pgdata, wal_path)
            .map_err(|err| WalHelperError::Message(err.to_string()))?;
    Ok(RenderedCommand {
        program: rendered.program,
        args: rendered.args,
    })
}

pub fn render_archive_get(
    pgdata: &Path,
    wal_segment: &str,
    destination_path: &str,
) -> Result<RenderedCommand, WalHelperError> {
    let rendered = crate::backup::archive_command::render_archive_get_from_pgdata(
        pgdata,
        wal_segment,
        destination_path,
    )
    .map_err(|err| WalHelperError::Message(err.to_string()))?;
    Ok(RenderedCommand {
        program: rendered.program,
        args: rendered.args,
    })
}
