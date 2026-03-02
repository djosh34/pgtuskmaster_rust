use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TlsMaterial {
    pub(crate) ca_cert: Option<PathBuf>,
    pub(crate) cert: Option<PathBuf>,
    pub(crate) key: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum TlsMode {
    #[default]
    Disabled,
    Optional,
    Required,
}
