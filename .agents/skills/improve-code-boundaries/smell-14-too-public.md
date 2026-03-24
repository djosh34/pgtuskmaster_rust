# Too public smell

Why are fields pub or pub(crate)? Does that make sense for this situation?
Or was it only done for testing purposes?


Please make as much fields, functions and anything as private as possible.
If those were only used for testing purposes, export them only via cfg entries.


Example:

These fields are only used within the same file: remove pub(crate) from them

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostgresConfig {
    pub(crate) postgresql_conf_path: PathBuf,
    pub(crate) hba_path: PathBuf,
    pub(crate) ident_path: PathBuf,
    pub(crate) standby_passfile_path: Option<PathBuf>,
    pub(crate) standby_signal_path: PathBuf,
    pub(crate) recovery_signal_path: PathBuf,
    pub(crate) postgresql_auto_conf_path: PathBuf,
    pub(crate) quarantined_postgresql_auto_conf_path: PathBuf,
}

```