#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FallbackClusterView {
    pub(crate) name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FallbackHeartbeatInput {
    pub(crate) source: String,
}
