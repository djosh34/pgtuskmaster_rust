pub type ApiRoleTokens = crate::config::RoleTokens;

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
