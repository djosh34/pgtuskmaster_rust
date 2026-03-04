use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BackupCommandTemplate {
    pub(crate) args: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BackupInput {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InfoInput {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CheckInput {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RestoreInput {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) pg1_path: PathBuf,
    pub(crate) options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArchivePushInput {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) pg1_path: PathBuf,
    pub(crate) wal_path: String,
    pub(crate) options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArchiveGetInput {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) pg1_path: PathBuf,
    pub(crate) wal_segment: String,
    pub(crate) destination_path: String,
    pub(crate) options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BackupOperation {
    Backup(BackupInput),
    Info(InfoInput),
    Check(CheckInput),
    Restore(RestoreInput),
    ArchivePush(ArchivePushInput),
    ArchiveGet(ArchiveGetInput),
}
