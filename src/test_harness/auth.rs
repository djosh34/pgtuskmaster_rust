#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AuthConfig {
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
}

impl AuthConfig {
    pub(crate) fn is_configured(&self) -> bool {
        self.username.is_some() && self.password.is_some()
    }
}
