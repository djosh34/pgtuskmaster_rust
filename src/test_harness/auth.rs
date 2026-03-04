use super::HarnessError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ApiRoleTokens {
    pub(crate) read_token: String,
    pub(crate) admin_token: String,
}

impl ApiRoleTokens {
    pub(crate) fn new(
        read_token: impl Into<String>,
        admin_token: impl Into<String>,
    ) -> Result<Self, HarnessError> {
        let read = read_token.into();
        let admin = admin_token.into();
        if read.trim().is_empty() {
            return Err(HarnessError::InvalidInput(
                "read role token must not be empty".to_string(),
            ));
        }
        if admin.trim().is_empty() {
            return Err(HarnessError::InvalidInput(
                "admin role token must not be empty".to_string(),
            ));
        }
        Ok(Self {
            read_token: read,
            admin_token: admin,
        })
    }

    pub(crate) fn read_bearer_header(&self) -> String {
        format!("Bearer {}", self.read_token)
    }

    pub(crate) fn admin_bearer_header(&self) -> String {
        format!("Bearer {}", self.admin_token)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_harness::HarnessError;

    use super::ApiRoleTokens;

    #[test]
    fn api_role_tokens_rejects_empty_input() {
        let result = ApiRoleTokens::new("", "admin");
        assert!(matches!(result, Err(HarnessError::InvalidInput(_))));
    }

    #[test]
    fn api_role_tokens_formats_bearer_headers() -> Result<(), HarnessError> {
        let tokens = ApiRoleTokens::new("read", "admin")?;
        assert_eq!(tokens.read_bearer_header(), "Bearer read");
        assert_eq!(tokens.admin_bearer_header(), "Bearer admin");
        Ok(())
    }
}
