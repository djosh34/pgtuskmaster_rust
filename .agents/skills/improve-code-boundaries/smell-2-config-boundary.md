# Smell 2: Wrong Config Boundary

This smell is about letting real-world config shapes leak into the rest of the program.

The correct shape is:

1. read raw config into a serde/TOML DTO
2. validate and normalize it once
3. convert it into a flatter internal Rust type
4. never validate the same thing again

After that conversion, invalid states should be unrepresentable.

That means:

- if a path must exist internally, do not keep it as `Option`
- if TLS is enabled, missing cert material must no longer be representable
- if inline content, file paths, or env variables are only ingestion concerns, downstream code must not care which source form produced the value

The raw input shape is allowed to be messy. The long-lived internal shape is not.

## Rules for this smell

- create one real-world DTO type that encompasses the external document shape
- keep it private or `pub(super)`
- do all validation and normalization at ingestion
- convert once
- expose only the validated shared type

If you see later code re-validating, re-resolving, or re-normalizing config, the boundary is wrong.

## Example A: source-world config types are exposed as shared runtime types

From `src/config/schema.rs`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum InlineOrPath {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SecretSource {
    #[default]
    None,
    Env { env: String },
    File { path: PathBuf },
    String { value: String },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum TlsServerConfig {
    #[default]
    Disabled,
    Enabled {
        identity: TlsServerIdentityConfig,
        client_auth: Option<TlsClientAuthConfig>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfig,
    pub dcs: DcsConfig,
    #[serde(default)]
    pub ha: HaConfig,
    #[serde(default)]
    pub process: ProcessConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub api: ApiConfig,
    pub pgtm: Option<PgtmConfig>,
    #[serde(default)]
    pub debug: DebugConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresPathsConfig {
    pub data_dir: PathBuf,
    pub socket_dir: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresClientTransportConfig {
    #[serde(default = "defaults::default_pg_ssl_mode")]
    pub ssl_mode: crate::pginfo::conninfo::PgSslMode,
    pub ca_cert: Option<InlineOrPath>,
}
```

This is still very close to the real-world input document:

- `InlineOrPath` is an ingestion concern
- `SecretSource` is an ingestion concern
- `Option<PathBuf>` fields may or may not be acceptable internally, depending on whether defaults are required
- `TlsServerConfig::Enabled { client_auth: Option<_> }` still allows several intermediate states that later code must interpret

These shapes are fine as raw DTOs. They are questionable as the long-lived shared runtime API.

## Example B: later code still resolves and validates source-world details

From `src/cli/config.rs`:

```rust
fn validate_effective_api_url(
    base_url: &Url,
    config_source: Option<&OperatorConfigSource>,
) -> Result<(), CliError> {
    let Some(source) = config_source else {
        return Ok(());
    };

    if let Some(expected_transport) = source
        .operator
        .as_ref()
        .and_then(|operator| operator.api.expected_transport)
    {
        return match (expected_transport, base_url.scheme()) {
            (PgtmApiTransportExpectation::Http, "https") => Err(CliError::Config(
                "API URL must not use https when `pgtm.api.expected_transport = \"http\"`"
                    .to_string(),
            )),
            (PgtmApiTransportExpectation::Https, "http") => Err(CliError::Config(
                "API URL must use https when `pgtm.api.expected_transport = \"https\"`".to_string(),
            )),
            _ => Ok(()),
        };
    }

    let Some(cfg) = source.runtime.as_ref() else {
        return Ok(());
    };

    match (&cfg.api.transport, base_url.scheme()) {
        (ApiTransportConfig::Http, "https") => Err(CliError::Config(
            "API URL must not use https when `api.transport = \"http\"`".to_string(),
        )),
        (ApiTransportConfig::Https { .. }, "http") => Err(CliError::Config(
            "API URL must use https when `api.transport = \"https\"`".to_string(),
        )),
        _ => Ok(()),
    }
}
```

```rust
fn resolve_api_client_tls(
    config_source: Option<&OperatorConfigSource>,
) -> Result<CliTlsConfig, CliError> {
    let Some(source) = config_source else {
        return Ok(CliTlsConfig::default());
    };
    let Some(api_client) = source.operator.as_ref().map(|operator| &operator.api.tls) else {
        return Ok(CliTlsConfig::default());
    };

    if api_requires_client_cert(source) && api_client.identity.is_none() {
        return Err(CliError::Config(
            "`pgtm.api.tls.identity` is required when API client certificates are mandatory"
                .to_string(),
        ));
    }

    Ok(CliTlsConfig {
        ca_cert_pem: api_client
            .ca_cert
            .as_ref()
            .map(|source| resolve_inline_or_path_bytes("pgtm.api.tls.ca_cert", source))
            .transpose()
            .map_err(|err| CliError::Config(err.to_string()))?,
        client_cert_pem: api_client
            .identity
            .as_ref()
            .map(|identity| {
                resolve_inline_or_path_bytes("pgtm.api.tls.identity.cert", &identity.cert)
            })
            .transpose()
            .map_err(|err| CliError::Config(err.to_string()))?,
        client_key_pem: api_client
            .identity
            .as_ref()
            .map(|identity| resolve_secret_string("pgtm.api.tls.identity.key", &identity.key))
            .transpose()
            .map(|result| result.map(String::into_bytes))
            .map_err(|err| CliError::Config(err.to_string()))?,
        ca_cert_path: api_client
            .ca_cert
            .as_ref()
            .and_then(inline_or_path_to_path_buf),
        client_cert_path: api_client
            .identity
            .as_ref()
            .and_then(|identity| inline_or_path_to_path_buf(&identity.cert)),
        client_key_path: api_client
            .identity
            .as_ref()
            .and_then(|identity| secret_to_path_buf(&identity.key)),
    })
}
```

```rust
fn inline_or_path_to_path_buf(source: &InlineOrPath) -> Option<PathBuf> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => Some(path.clone()),
        InlineOrPath::Inline { .. } => None,
    }
}

fn secret_to_path_buf(source: &SecretSource) -> Option<PathBuf> {
    match source {
        SecretSource::File { path } => Some(path.clone()),
        SecretSource::None | SecretSource::Env { .. } | SecretSource::String { .. } => None,
    }
}
```

Why this is the smell:

- later code still cares whether data came from inline content or a path
- later code still cares whether a secret came from a file, env var, or inline string
- later code still checks transport and TLS compatibility
- later code still performs fallback logic from API TLS to Postgres TLS

That is proof the ingestion boundary did not finish its job once.

## Example C: parse and validate are separate, but the shared type is still the raw shape

From `src/config/parser.rs`:

```rust
pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfig, ConfigError> {
    let contents = read_config_file(path)?;
    let cfg: RuntimeConfig = toml::from_str(&contents).map_err(|source| ConfigError::Parse {
        path: path.display().to_string(),
        source,
    })?;
    validate_runtime_config(&cfg)?;
    Ok(cfg)
}
```

The issue is not that validation exists. The issue is that the validated result is still the same broad serde-shaped `RuntimeConfig`.

That means later code keeps carrying around:

- input-source distinctions
- option-heavy shapes
- normalization work

## How to untangle smell 2

1. Identify the true external boundary DTO.
   In this repo that is the TOML-deserialized config document shape.
2. Make that DTO private or `pub(super)`.
3. Temporarily block downstream direct use of that DTO type and run `make check`.
4. Let each failure tell you what later code was still doing with config:
   - validating
   - resolving inline vs path
   - resolving secrets
   - applying defaults or inheritance
   - checking transport or TLS consistency
5. Move each of those behaviors back into config ingestion.
6. Create one flatter internal type where only valid cases exist.
7. Remove leftover helper functions whose only job was to normalize the old source-world shape.
8. Keep iterating until downstream code directly matches on the validated internal enum or struct.

## What the flatter internal type should do

The internal shared type should encode guarantees, for example:

- TLS disabled
- TLS enabled with fully resolved server identity
- client auth optional with resolved CA
- client auth required with resolved CA and any additional validated constraints

Likewise:

- a required path should already be present
- a usable certificate or key should already be resolved
- later code should not ask "was this inline or a file?"

If that source distinction still matters later, your boundary is still wrong.

## Decision rule for smell 2

If a piece of later code is doing something that could have been known when reading config, move it earlier.

Do not stop after moving one validation. Keep following every use of the DTO-derived shape. Many functions on the old type become useless once the validated internal type exists. Remove them. Ten or twenty touched files is acceptable if that is what it takes to make the boundary honest.

## Exceptions

Real-world input and output payloads are allowed to exist. The rule is not "no DTOs." The rule is:

- DTOs stay at the edge
- DTOs stay private
- validation and normalization happen once
- the rest of the code never re-enters the raw world

