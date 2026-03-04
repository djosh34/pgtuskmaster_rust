use std::path::{Path, PathBuf};

use crate::config::RuntimeConfig;
use crate::state::WorkerError;

pub(crate) const DEFAULT_ARCHIVE_WRAPPER_NAME: &str = "pgtuskmaster-archive-wrapper.sh";

pub(crate) fn ensure_archive_wrapper(cfg: &RuntimeConfig) -> Result<Option<PathBuf>, WorkerError> {
    let Some(log_file) = cfg.logging.postgres.archive_command_log_file.as_ref() else {
        return Ok(None);
    };

    let parent = log_file.parent().ok_or_else(|| {
        WorkerError::Message(format!(
            "archive_command_log_file has no parent directory: {}",
            log_file.display()
        ))
    })?;

    let script_path = parent.join(DEFAULT_ARCHIVE_WRAPPER_NAME);
    write_archive_wrapper_script(script_path.as_path(), log_file.as_path())?;
    Ok(Some(script_path))
}

fn write_archive_wrapper_script(script_path: &Path, log_file: &Path) -> Result<(), WorkerError> {
    let script = format!(
        r#"#!/bin/sh
set -eu

ARCHIVE_DIR=""
if [ "${{1:-}}" = "--archive-dir" ]; then
  ARCHIVE_DIR="${{2:-}}"
  shift 2
fi

SRC="${{1:-}}"
FILENAME="${{2:-}}"

if [ -z "${{ARCHIVE_DIR}}" ]; then
  ARCHIVE_DIR="${{PGTUSKMASTER_ARCHIVE_DIR:-}}"
fi

if [ -z "${{SRC}}" ] || [ -z "${{FILENAME}}" ]; then
  echo "usage: $0 [--archive-dir DIR] <src> <filename>" >&2
  exit 2
fi

if [ -z "${{ARCHIVE_DIR}}" ]; then
  echo "missing archive dir (pass --archive-dir or set PGTUSKMASTER_ARCHIVE_DIR)" >&2
  exit 2
fi

mkdir -p "${{ARCHIVE_DIR}}"
DST="${{ARCHIVE_DIR}}/${{FILENAME}}"

OUTPUT=""
if OUTPUT="$(cp "${{SRC}}" "${{DST}}" 2>&1)"; then
  STATUS=0
else
  STATUS="$?"
fi

export PGTUSKMASTER_ARCHIVE_STATUS="$STATUS"
export PGTUSKMASTER_ARCHIVE_SRC="$SRC"
export PGTUSKMASTER_ARCHIVE_DST="$DST"
MESSAGE="archive_command status=$STATUS src=$SRC dst=$DST"
if [ -n "${{OUTPUT}}" ]; then
  MESSAGE="${{MESSAGE}}
${{OUTPUT}}"
fi
export PGTUSKMASTER_ARCHIVE_MESSAGE="$MESSAGE"

python3 - << 'PY' >> "{log_file}"
import json
import os
import sys

status = int(os.environ.get("PGTUSKMASTER_ARCHIVE_STATUS", "0"))

record = {{
  "severity": "LOG" if status == 0 else "ERROR",
  "message": os.environ.get("PGTUSKMASTER_ARCHIVE_MESSAGE", ""),
  "pgtuskmaster": {{
    "archive": {{
      "status": status,
      "src": os.environ.get("PGTUSKMASTER_ARCHIVE_SRC", ""),
      "dst": os.environ.get("PGTUSKMASTER_ARCHIVE_DST", ""),
    }}
  }}
}}
sys.stdout.write(json.dumps(record, separators=(",", ":")) + "\n")
PY

exit "${{STATUS}}"
"#,
        log_file = log_file.display(),
    );

    std::fs::write(script_path, script)
        .map_err(|err| WorkerError::Message(format!("write archive wrapper failed: {err}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(script_path, perms).map_err(|err| {
            WorkerError::Message(format!("chmod archive wrapper failed: {err}"))
        })?;
    }

    Ok(())
}
