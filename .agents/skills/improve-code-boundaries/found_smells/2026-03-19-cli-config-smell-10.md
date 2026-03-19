path: /home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/config.rs 31-377

- I found smell 10
since it looks like
```rust
pub(crate) fn resolve_operator_context(cli: &Cli) -> Result<OperatorContext, CliError> {
    let config_source = cli
        .config
        .as_ref()
        .map(|path| load_operator_config_source(path.as_path()))
        .transpose()?;

    let base_url = resolve_api_url(cli.base_url.as_deref(), config_source.as_ref())?;
    validate_effective_api_url(&base_url, config_source.as_ref())?;

    let (config_read_token, config_admin_token, api_auth_enabled) =
        resolve_config_auth(config_source.as_ref())?;
    let read_token = normalize_optional_token(cli.read_token.as_deref()).or(config_read_token);
    let admin_token = normalize_optional_token(cli.admin_token.as_deref()).or(config_admin_token);

    let api_client_tls = if base_url.scheme() == "https" {
        resolve_api_client_tls(config_source.as_ref())?
    } else {
        CliTlsConfig::default()
    };
    let postgres_client_tls = resolve_postgres_client_tls(config_source.as_ref())?;
}

fn resolve_api_url(
    override_base_url: Option<&str>,
    config_source: Option<&OperatorConfigSource>,
) -> Result<Url, CliError> { /* one caller */ }

fn validate_effective_api_url(
    base_url: &Url,
    config_source: Option<&OperatorConfigSource>,
) -> Result<(), CliError> { /* one caller */ }

fn resolve_config_auth(
    config_source: Option<&OperatorConfigSource>,
) -> Result<(Option<String>, Option<String>, bool), CliError> { /* one caller */ }

fn resolve_api_client_tls(
    config_source: Option<&OperatorConfigSource>,
) -> Result<CliTlsConfig, CliError> { /* one caller */ }

fn resolve_postgres_client_tls(
    config_source: Option<&OperatorConfigSource>,
) -> Result<CliTlsConfig, CliError> { /* one caller */ }
```
