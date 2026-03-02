#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsKey {
    Member(String),
    Leader,
    Switchover,
    Config,
    InitLock,
}
