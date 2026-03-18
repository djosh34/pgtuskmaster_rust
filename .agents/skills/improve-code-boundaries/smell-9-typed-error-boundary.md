# Smell 9: Typed Error Boundary, Not String Buckets

This smell is about one very specific reduction:

- before: internal code keeps doing `map_err(|err| ... err.to_string())`, `map_err(|err| format!(...))`, or `Err(format!(...))`
- after: internal code returns typed errors, the error enum carries the real source with `thiserror`, and the application code gets much smaller

Read every example below literally:

- `Before` means the shape that exists in this repo today
- `After` means the target shape we want after the boundary is cleaned up
- the win is not abstract "better typing"
- the win is that the string-building `map_err` glue mostly disappears

If code is still inside the program, it should usually return a typed error. Human prose belongs at the outer display/log/HTTP boundary, not in the middle of control flow.

## Core Rule

Keep the source error typed until the edge.

That usually means:

1. return `Result<T, FooError>`, not `Result<T, String>`
2. store the real source error in the enum
3. use `#[from]` when one source maps cleanly to one variant
4. use `#[source]` plus context fields when the same source type can fail in several operations
5. render once at the final boundary that talks to a human

The important distinction is:

- `#[from]` removes the conversion completely when the mapping is one-to-one
- `#[source]` still keeps the real error typed even when you need operation-specific context
- both are better than `err.to_string()`

Do not stop thinking too early:

- if `map_err(|source| FooError::Bar { source })` is still present, that may be acceptable as a temporary reduction
- but it may also mean the boundary is still wrong
- very often the real fix is to move the semantic distinction into the error type or into smaller helper boundaries so the caller can use `?` all the way through

## Detection Checklist

Look for these signals:

- `Result<_, String>` in internal helpers or modules
- error enums with `Message(String)`, `Config(String)`, `Transport(String)`, `Decode(String)`, `Failed(String)`, or similar string buckets
- `map_err(|err| err.to_string())`
- `map_err(|err| format!(...))`
- `Err(format!(...))`
- `Vec<String>` used to accumulate failures that could stay typed
- a typed source error already exists, but the caller immediately flattens it into text
- repeated `map_err` closures whose only job is to build another sentence

## Before And After Must Be Obvious

When you document or review this smell, always show all four pieces together:

1. the `Before` application code
2. the `Before` error type
3. the `After` application code
4. the `After` error type

If you skip either the application code or the error type, the example becomes fuzzy.

## Example 1: `src/api/worker.rs`

This is the smallest complete example in the repo.

### Before: actual error type in the repo today

```rust
#[derive(Debug, thiserror::Error)]
enum ReloadCertificatesError {
    #[error("api certificate reload failed: {message}")]
    Api { message: String },
    #[error("postgres certificate reload failed: {0}")]
    Postgres(#[from] crate::process::postmaster::ManagedPostmasterError),
}
```

### Before: actual application code in the repo today

```rust
async fn reload(
    &self,
    cfg: &RuntimeConfig,
) -> Result<ApiCertificateReloadStep, ReloadCertificatesError> {
    match self {
        Self::HttpTransport => Ok(ApiCertificateReloadStep::HttpTransportUnchanged),
        Self::Https { server_config } => match &cfg.api.transport {
            crate::config::ApiTransportConfig::Http => Err(ReloadCertificatesError::Api {
                message: "api cert reload requires https transport".to_string(),
            }),
            crate::config::ApiTransportConfig::Https { tls } => {
                let reloaded = crate::tls::build_api_server_config(tls).map_err(|err| {
                    ReloadCertificatesError::Api {
                        message: err.to_string(),
                    }
                })?;
                server_config.reload_from_config(reloaded);
                Ok(ApiCertificateReloadStep::HttpsConfigurationReloaded)
            }
        },
    }
}
```

### After: target error type

```rust
#[derive(Debug, thiserror::Error)]
enum ReloadCertificatesError {
    #[error("api certificate reload requires https transport")]
    ApiTransportMismatch,
    #[error(transparent)]
    ApiTls(#[from] crate::tls::TlsConfigError),
    #[error("postgres certificate reload failed: {0}")]
    Postgres(#[from] crate::process::postmaster::ManagedPostmasterError),
}
```

### After: target application code

```rust
async fn reload(
    &self,
    cfg: &RuntimeConfig,
) -> Result<ApiCertificateReloadStep, ReloadCertificatesError> {
    match self {
        Self::HttpTransport => Ok(ApiCertificateReloadStep::HttpTransportUnchanged),
        Self::Https { server_config } => match &cfg.api.transport {
            crate::config::ApiTransportConfig::Http => {
                Err(ReloadCertificatesError::ApiTransportMismatch)
            }
            crate::config::ApiTransportConfig::Https { tls } => {
                let reloaded = crate::tls::build_api_server_config(tls)?;
                server_config.reload_from_config(reloaded);
                Ok(ApiCertificateReloadStep::HttpsConfigurationReloaded)
            }
        },
    }
}
```

### What got smaller

- `map_err(|err| ReloadCertificatesError::Api { message: err.to_string() })` disappeared
- `TlsConfigError` stays typed instead of being flattened into a sentence
- the application code now uses plain `?`
- this is the ideal smell-9 cleanup: less code and better information

## Example 2: `tests/ha/support/invariants/write_convergence.rs`

This is a much stronger smell-9 example because the problem is not just `map_err`.

The real problem is:

- the observation type stores `message: String`
- the function keeps merging error text with `format!(...)`
- `previous_error` is already flattened to `Option<String>`
- once the code does that, every new failure path has to become more string glue

### Before: actual error types in the repo today

```rust
#[derive(Debug, thiserror::Error)]
pub enum WriteConvergenceInvariantError {
    #[error("write-convergence invariant failed: {0}")]
    Failed(String),
}

enum MemberCountObservation {
    Observed {
        member: ClusterMember,
        count: u64,
    },
    Failed {
        member: ClusterMember,
        message: String,
    },
}
```

### Before: actual application code in the repo today

```rust
async fn read_member_count_via_fresh_connection(
    member: &MemberObservationTarget,
    connect_timeout: Duration,
    previous_error: Option<String>,
) -> MemberCountObservation {
    let routing_target = match resolve_observation_routing_target(member) {
        Ok(routing_target) => routing_target,
        Err(err) => {
            return MemberCountObservation::Failed {
                member: member.routing_target.member,
                message: previous_error.map_or(err.clone(), |previous| {
                    format!(
                        "existing observation failed: {previous}; refresh routing failed: {err}"
                    )
                }),
            };
        }
    };
    match connect_member(&routing_target, connect_timeout).await {
        Ok((client, connection_task)) => {
            let count_result = read_count(client.as_ref(), connect_timeout).await;
            connection_task.abort();
            match count_result {
                Ok(count) => MemberCountObservation::Observed {
                    member: member.routing_target.member,
                    count,
                },
                Err(err) => MemberCountObservation::Failed {
                    member: member.routing_target.member,
                    message: previous_error.map_or_else(
                        || err.to_string(),
                        |previous| format!(
                            "existing observation failed: {previous}; fresh reconnect read failed: {err}"
                        ),
                    ),
                },
            }
        }
        Err(err) => MemberCountObservation::Failed {
            member: member.routing_target.member,
            message: previous_error.map_or_else(
                || err.clone(),
                |previous| {
                    format!(
                        "existing observation failed: {previous}; fresh reconnect failed: {err}"
                    )
                },
            ),
        },
    }
}
```

### Why this before is bad

- the failure boundary is already `String`
- `previous_error` is no longer a cause, it is just prose
- the code cannot use `#[from]` because the helpers do not return operation-specific typed errors
- every branch is forced to build sentences manually

This is exactly the wrong instinct:

- flatten first
- then concatenate more text on top
- then try to explain the mess with more formatting

### After: much cleaner error types

```rust
#[derive(Debug, thiserror::Error)]
#[error("refresh routing failed")]
struct RefreshRoutingError(#[from] crate::support::error::HarnessError);

#[derive(Debug, thiserror::Error)]
#[error("fresh reconnect failed")]
struct FreshReconnectError(#[from] ConnectMemberError);

#[derive(Debug, thiserror::Error)]
#[error("fresh reconnect read failed")]
struct FreshReconnectReadError(#[from] ReadCountError);

#[derive(Debug, thiserror::Error)]
enum MemberCountObservationError {
    #[error(transparent)]
    RefreshRouting(#[from] RefreshRoutingError),

    #[error(transparent)]
    FreshReconnect(#[from] FreshReconnectError),

    #[error(transparent)]
    FreshReconnectRead(#[from] FreshReconnectReadError),

    #[error("existing observation failed, and refresh observation also failed")]
    ExistingAndFresh {
        #[source]
        previous: Box<MemberCountObservationError>,
        #[source]
        fresh: Box<MemberCountObservationError>,
    },
}

enum MemberCountObservation {
    Observed {
        member: ClusterMember,
        count: u64,
    },
    Failed {
        member: ClusterMember,
        error: MemberCountObservationError,
    },
}

impl MemberCountObservation {
    fn render(&self) -> String {
        match self {
            Self::Observed { member, count } => format!("`{member}`={count}"),
            Self::Failed { member, error } => format!("`{member}` error={error}"),
        }
    }
}
```

### After: much cleaner application code

```rust
fn resolve_observation_routing_target(
    member: &MemberObservationTarget,
) -> Result<PostgresRoutingTarget, RefreshRoutingError> {
    Ok(member.observer.postgres_routing_target(member.routing_target.member)?)
}

async fn reconnect_and_read_member_count(
    member: &MemberObservationTarget,
    connect_timeout: Duration,
) -> Result<u64, MemberCountObservationError> {
    let routing_target = resolve_observation_routing_target(member)?;
    let (client, connection_task) = connect_member(&routing_target, connect_timeout).await?;
    let count = read_count(client.as_ref(), connect_timeout).await?;
    connection_task.abort();
    Ok(count)
}

async fn read_member_count_via_fresh_connection(
    member: &MemberObservationTarget,
    connect_timeout: Duration,
    previous_error: Option<MemberCountObservationError>,
) -> MemberCountObservation {
    match reconnect_and_read_member_count(member, connect_timeout).await {
        Ok(count) => MemberCountObservation::Observed {
            member: member.routing_target.member,
            count,
        },
        Err(fresh) => MemberCountObservation::Failed {
            member: member.routing_target.member,
            error: previous_error.map_or(fresh, |previous| {
                MemberCountObservationError::ExistingAndFresh {
                    previous: Box::new(previous),
                    fresh: Box::new(fresh),
                }
            }),
        },
    }
}
```

### What got much cleaner

- the top-level refresh flow no longer has any `map_err`
- the top-level refresh flow no longer has any `to_string()`
- the top-level refresh flow no longer has any `format!(...)`
- the observation keeps a typed `error`, not a `message: String`
- `#[from]` works because each helper now owns one semantic operation and one error type
- the human-readable sentence is rendered once in `MemberCountObservation::render`

This is the important challenge:

- no, you are not stuck with typed `map_err` slop forever
- if the code still needs it everywhere, the boundary is probably still wrong
- the real move is often to split one multi-step function into smaller typed operations so `#[from]` can collapse the glue all at once

## Example 3: `src/cli/error.rs` and `src/cli/client.rs`

This is a string-bucket boundary. The enum is typed only in name. The payloads are still strings.

### Before: actual error type in the repo today

```rust
#[derive(Debug, Error)]
pub enum CliError {
    #[error("config error: {0}")]
    Config(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("api request failed with status {status}: {body}")]
    ApiStatus { status: u16, body: String },
    #[error("response decode failed: {0}")]
    Decode(String),
    #[error("request build failed: {0}")]
    RequestBuild(String),
    #[error("resolution error: {0}")]
    Resolution(String),
    #[error("output serialization failed: {0}")]
    Output(String),
}
```

### Before: actual application code in the repo today

```rust
let url = self
    .base_url
    .join(path)
    .map_err(|err| CliError::RequestBuild(format!("compose URL failed: {err}")))?;

let response = request
    .send()
    .await
    .map_err(|err| CliError::Transport(err.to_string()))?;

let body = response
    .text()
    .await
    .map_err(|err| CliError::Transport(err.to_string()))?;

serde_json::from_str(&body).map_err(|err| CliError::Decode(err.to_string()))
```

### After: target error type

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Config(#[from] CliConfigError),

    #[error("api request failed with status {status}: {body}")]
    ApiStatus { status: u16, body: String },

    #[error(transparent)]
    Transport(#[from] reqwest::Error),

    #[error("compose URL for `{path}` failed")]
    ComposeUrl {
        path: &'static str,
        #[source]
        source: url::ParseError,
    },

    #[error(transparent)]
    Decode(#[from] serde_json::Error),

    #[error("parse CA certificate failed")]
    ParseCaCert {
        #[source]
        source: reqwest::Error,
    },

    #[error("parse client identity failed")]
    ParseClientIdentity {
        #[source]
        source: reqwest::Error,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum CliConfigError {
    #[error("load operator config failed")]
    LoadOperatorConfig(#[from] crate::config::ConfigError),

    #[error("read CLI config file {path}")]
    ReadConfigFile {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("materialize `{field}` failed")]
    Materialize {
        field: &'static str,
        #[source]
        source: crate::config::ConfigMaterializeError,
    },
}
```

### After: target application code

```rust
let url = self
    .base_url
    .join(path)
    .map_err(|source| CliError::ComposeUrl { path, source })?;

let response = request.send().await?;
let body = response.text().await?;

serde_json::from_str(&body).map_err(CliError::from)
```

### What got smaller

- `Transport(err.to_string())` disappeared
- `Decode(err.to_string())` disappeared
- the enum now carries `reqwest::Error`, `url::ParseError`, and `serde_json::Error`
- the call sites use `?` again instead of text conversion closures

## What Good Looks Like

A good smell-9 boundary usually has one of these shapes:

```rust
#[derive(Debug, thiserror::Error)]
enum FooError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("parse `{field}` failed")]
    Parse {
        field: &'static str,
        #[source]
        source: ParseError,
    },
}
```

```rust
#[derive(Debug, thiserror::Error)]
enum BarError {
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),

    #[error(transparent)]
    Worker(#[from] WorkerError),
}
```

The key is not the exact variant names.

The key is:

- source stays typed
- `?` becomes possible again
- `map_err(|err| err.to_string())` disappears
- rendering happens once at the edge

And one more rule:

- if a typed `map_err` is still present, ask whether it is describing a real boundary or compensating for a badly-shaped one

## How To Untangle Smell 9

1. Find the first place where a typed error becomes `String`.
2. Replace the string bucket with a typed enum variant.
3. If one source maps cleanly to one variant, add `#[from]`.
4. If the same source type can fail in different operations, keep it typed with `#[source]` and add context fields or operation-specific variants.
5. Rewrite the call site so `?` handles the one-to-one cases.
6. Keep the remaining conversions small and typed.
7. Render the final message only at the human-facing boundary.
8. Re-run `make check`.

## Preferred Repo Direction

In this repo, prefer:

- typed source errors in `src/`
- typed harness errors in `tests/ha/support/`
- `#[from]` wherever the mapping is direct
- `#[source]` where extra context is needed
- `thiserror` instead of manual string buckets
- `String` only at the real presentation boundary

Avoid:

- `Result<_, String>` in internal code
- `Message(String)` as the default answer to every failure
- repeated `map_err(|err| err.to_string())`
- repeated `map_err(|err| format!(...))`
- converting to prose in the middle and then pretending the error is still typed

This is related to smell 4, but different:

- smell 4 is about rendering output too early
- smell 9 is about flattening errors too early
