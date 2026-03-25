#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiRoleTokens {
    pub read_token: String,
    pub admin_token: String,
}

impl ApiRoleTokens {
    pub fn new(
        read_token: impl Into<String>,
        admin_token: impl Into<String>,
    ) -> Result<Self, String> {
        let read_token = read_token.into();
        let admin_token = admin_token.into();
        if read_token.trim().is_empty() {
            return Err("read token must not be empty".to_string());
        }
        if admin_token.trim().is_empty() {
            return Err("admin token must not be empty".to_string());
        }
        Ok(Self {
            read_token,
            admin_token,
        })
    }

    pub fn read_bearer_header(&self) -> String {
        format!("Bearer {}", self.read_token)
    }

    pub fn admin_bearer_header(&self) -> String {
        format!("Bearer {}", self.admin_token)
    }
}

#[cfg(test)]
mod tests {
    use super::ApiRoleTokens;

    #[test]
    fn api_role_tokens_rejects_empty_input() {
        let result = ApiRoleTokens::new("", "admin");
        assert!(result.is_err());
    }

    #[test]
    fn api_role_tokens_formats_bearer_headers() -> Result<(), String> {
        let tokens = ApiRoleTokens::new("read", "admin")?;
        assert_eq!(tokens.read_bearer_header(), "Bearer read");
        assert_eq!(tokens.admin_bearer_header(), "Bearer admin");
        Ok(())
    }
}
