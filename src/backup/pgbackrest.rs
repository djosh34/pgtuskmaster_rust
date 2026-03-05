use crate::backup::{
    ArchiveGetInput, ArchivePushInput, BackupCommandTemplate, BackupInput, BackupOperation,
    CheckInput, InfoInput, RestoreInput,
};

pub(crate) fn render(operation: BackupOperation) -> Result<BackupCommandTemplate, String> {
    match operation {
        BackupOperation::Backup(input) => render_backup(input),
        BackupOperation::Info(input) => render_info(input),
        BackupOperation::Check(input) => render_check(input),
        BackupOperation::Restore(input) => render_restore(input),
        BackupOperation::ArchivePush(input) => render_archive_push(input),
        BackupOperation::ArchiveGet(input) => render_archive_get(input),
    }
}

fn render_backup(input: BackupInput) -> Result<BackupCommandTemplate, String> {
    validate_non_empty("stanza", input.stanza.as_str())?;
    validate_non_empty("repo", input.repo.as_str())?;
    validate_option_tokens(&input.options)?;

    let mut args = base_args(input.stanza.as_str(), input.repo.as_str());
    args.extend(input.options);
    args.push("backup".to_string());
    Ok(BackupCommandTemplate { args })
}

fn render_info(input: InfoInput) -> Result<BackupCommandTemplate, String> {
    validate_non_empty("stanza", input.stanza.as_str())?;
    validate_non_empty("repo", input.repo.as_str())?;
    validate_option_tokens(&input.options)?;

    let mut args = base_args(input.stanza.as_str(), input.repo.as_str());
    args.extend(input.options);
    args.push("--output=json".to_string());
    args.push("info".to_string());
    Ok(BackupCommandTemplate { args })
}

fn render_check(input: CheckInput) -> Result<BackupCommandTemplate, String> {
    validate_non_empty("stanza", input.stanza.as_str())?;
    validate_non_empty("repo", input.repo.as_str())?;
    validate_option_tokens(&input.options)?;

    let mut args = base_args(input.stanza.as_str(), input.repo.as_str());
    args.extend(input.options);
    args.push("--output=json".to_string());
    args.push("check".to_string());
    Ok(BackupCommandTemplate { args })
}

fn render_restore(input: RestoreInput) -> Result<BackupCommandTemplate, String> {
    validate_non_empty("stanza", input.stanza.as_str())?;
    validate_non_empty("repo", input.repo.as_str())?;
    validate_non_empty_path("pg1_path", &input.pg1_path)?;
    validate_option_tokens(&input.options)?;

    let mut args = base_args(input.stanza.as_str(), input.repo.as_str());
    args.push("--pg1-path".to_string());
    args.push(input.pg1_path.display().to_string());
    args.extend(input.options);
    args.push("restore".to_string());
    Ok(BackupCommandTemplate { args })
}

fn render_archive_push(input: ArchivePushInput) -> Result<BackupCommandTemplate, String> {
    validate_non_empty("stanza", input.stanza.as_str())?;
    validate_non_empty("repo", input.repo.as_str())?;
    validate_non_empty_path("pg1_path", &input.pg1_path)?;
    validate_non_empty("wal_path", input.wal_path.as_str())?;
    validate_option_tokens(&input.options)?;

    // `archive-push` does not accept `--repo` (pgBackRest selects the repo via `--repo*-path`
    // configuration/options instead), so do not include it even though we keep `repo` in our
    // config for consistency with other operations.
    let mut args = archive_base_args(input.stanza.as_str());
    args.push("--pg1-path".to_string());
    args.push(input.pg1_path.display().to_string());
    args.extend(input.options);
    args.push("archive-push".to_string());
    args.push(input.wal_path);
    Ok(BackupCommandTemplate { args })
}

fn render_archive_get(input: ArchiveGetInput) -> Result<BackupCommandTemplate, String> {
    validate_non_empty("stanza", input.stanza.as_str())?;
    validate_non_empty("repo", input.repo.as_str())?;
    validate_non_empty_path("pg1_path", &input.pg1_path)?;
    validate_non_empty("wal_segment", input.wal_segment.as_str())?;
    validate_non_empty("destination_path", input.destination_path.as_str())?;
    validate_option_tokens(&input.options)?;

    // `archive-get` does not accept `--repo` (pgBackRest selects the repo via `--repo*-path`
    // configuration/options instead), so do not include it even though we keep `repo` in our
    // config for consistency with other operations.
    let mut args = archive_base_args(input.stanza.as_str());
    args.push("--pg1-path".to_string());
    args.push(input.pg1_path.display().to_string());
    args.extend(input.options);
    args.push("archive-get".to_string());
    args.push(input.wal_segment);
    args.push(input.destination_path);
    Ok(BackupCommandTemplate { args })
}

fn base_args(stanza: &str, repo: &str) -> Vec<String> {
    vec![
        "--stanza".to_string(),
        stanza.to_string(),
        "--repo".to_string(),
        repo.to_string(),
    ]
}

fn archive_base_args(stanza: &str) -> Vec<String> {
    vec!["--stanza".to_string(), stanza.to_string()]
}

fn validate_non_empty(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(())
}

fn validate_non_empty_path(field: &str, value: &std::path::Path) -> Result<(), String> {
    if value.as_os_str().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(())
}

fn validate_option_tokens(tokens: &[String]) -> Result<(), String> {
    for token in tokens {
        if token.trim().is_empty() {
            return Err("options must not contain empty tokens".to_string());
        }
        if token
            .as_bytes()
            .iter()
            .any(|b| matches!(b, 0 | b'\n' | b'\r'))
        {
            return Err("options must not contain NUL/newline characters".to_string());
        }
        let trimmed = token.trim_start();
        let key = option_key(trimmed);
        if matches!(key, "--stanza" | "--repo" | "--pg1-path") {
            return Err("options must not override managed fields (stanza/repo/pg1-path)".to_string());
        }
    }
    Ok(())
}

fn option_key(token: &str) -> &str {
    let Some(eq) = token.find('=') else {
        return token;
    };
    &token[..eq]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::provider::{BackupInput, InfoInput};

    #[test]
    fn render_info_includes_output_json_and_command() -> Result<(), String> {
        let out = render_info(InfoInput {
            stanza: "stanza-a".to_string(),
            repo: "1".to_string(),
            options: vec!["--log-level-console=info".to_string()],
        })?;
        assert!(out.args.contains(&"--output=json".to_string()));
        assert_eq!(out.args.last().map(String::as_str), Some("info"));
        Ok(())
    }

    #[test]
    fn render_backup_forbids_stanza_override_in_options() {
        let out = render_backup(BackupInput {
            stanza: "stanza-a".to_string(),
            repo: "1".to_string(),
            options: vec!["--stanza=evil".to_string()],
        });
        assert!(out.is_err());
    }
}
