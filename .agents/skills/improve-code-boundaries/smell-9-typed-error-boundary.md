# Smell 9: Typed Error Boundary, Not String Buckets

This smell is about preserving the real error shape until the outermost boundary instead of converting everything to `String` early.

The preferred shape is:

1. keep source errors typed
2. add context with struct variants, not sentence strings
3. use `#[from]` or manual `From` impls where a source maps cleanly to one variant
4. render only once at the edge that talks to humans

Do not return `Result<_, String>` if the failure has real structure. Do not build error prose in the middle of the control flow if the caller may need to branch on the cause.

## Detection Checklist

Look for these signals:

- `Result<_, String>` return types in helpers or internal modules
- enum variants like `Message(String)`, `Error(String)`, `Failed(String)`, `InvalidInput(String)`, `Resolution(String)`, or `Transport(String)` that hold no real invariant
- `map_err(|err| err.to_string())`, `format!("{err}")`, or `format!(...)` used before the final display/log boundary
- `Vec<String>` aggregating failures that could stay typed
- `thiserror` types that exist, but their fields are still `String` instead of source errors or structured context
- one helper translating several distinct failures into one generic variant
- code that re-parses message text later because the original cause was discarded

## What Good Looks Like

A good error boundary usually has one of these shapes:

- `#[derive(thiserror::Error)] enum FooError { #[error("...")] Io(#[from] std::io::Error), #[error("...")] Parse { field: String, #[source] source: ParseError } }`
- one small enum per module, with `#[from]` for the sources that map cleanly
- contextual fields like `path: PathBuf`, `member: MemberId`, `field: &'static str`, `op: &'static str`, or `status: u16`
- one final rendering edge for CLI output, logs, or HTTP responses

## Example A: The Repo Already Has Typed Error Boundaries

The repo already has several good examples to copy:

- `src/dev_support/mod.rs` uses distinct machine-readable variants like `Io`, `SpawnFailure`, `StartupTimeout`, `EarlyExit`, `ShutdownTimeout`, and `StalePath`
- `src/process/postmaster.rs` keeps path, pid, and signal context as data instead of collapsing them into prose
- `src/config/parser.rs` keeps `Io` and `Parse` typed at the ingestion edge
- `src/logging/core/runtime.rs` keeps separate error families instead of one generic bucket
- `tests/ha/support/error.rs` is a presentation boundary where `Message(String)` can remain the exception

The rule to copy is the boundary shape, not the exact variant names: preserve the source error and structured context until the outermost display boundary.

Real repo examples to copy from:

- `src/process/postmaster.rs` already models failures as real variants:

```rust
ReadPidFile { pid_file: PathBuf, message: String },
InvalidPid { pid_file: PathBuf, value: String, message: String },
SignalDelivery { pid: u32, signal: &'static str, message: String },
```

- `src/config/parser.rs` already keeps config failures typed:

```rust
Io { path: String, #[source] source: std::io::Error },
Parse { path: String, #[source] source: toml::de::Error },
Validation { field: &'static str, message: String },
```

- `src/logging/core/runtime.rs` already splits the internal logging error families:

```rust
Json(String),
SinkIo(String),
FileSinkInit { path: PathBuf, cause: String },
```

- `src/pginfo/conninfo.rs` is still stringly at the parse edge:

```rust
type Err = String;
```

- `tests/ha/support/invariants/write_convergence.rs` collapses distinct failures into one bucket:

```rust
Failed(String),
```

- `src/ha/worker.rs` is a live orchestration example of the anti-pattern:

```rust
changed.map_err(|err| WorkerError::Message(format!("ha pg subscriber closed: {err}")))?;
```

## Example B: The Wrong Shape Is Stringly From the Start

A smell usually looks like this:

```rust
pub enum WorkerError {
    Message(String),
}
```

or:

```rust
fn resolve_thing(...) -> Result<T, String> {
    source.map_err(|err| err.to_string())?;
    Err(format!("failed to resolve {name}"))
}
```

That is a signal that the code is losing the distinction between:

- source failure
- validation failure
- transport failure
- timeout
- invariant violation

If callers need to branch on those causes later, the string bucket is the wrong boundary.

## Example C: Structured Context Beats Sentence Strings

Prefer this:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file failed at {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid url in {field}")]
    Url {
        field: &'static str,
        #[source]
        source: url::ParseError,
    },
}
```

over this:

```rust
Err(format!("invalid url in {field}: {err}"))
```

The first version keeps the data usable for logging, retries, status mapping, and tests. The second one forces text parsing if anyone wants the cause.

## Example D: How `#[from]` Removes Boilerplate

Once you have a typed variant, `#[from]` lets the helper return the source error directly:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file failed at {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid url in {field}")]
    Url {
        field: &'static str,
        #[source]
        source: url::ParseError,
    },
}

fn load_config(path: &Path) -> Result<String, ConfigError> {
    let raw = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(raw)
}
```

If the source maps one-to-one to the variant, `#[from]` removes even more glue:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("worker failed")]
    Worker(#[from] WorkerError),
}

fn run_worker() -> Result<(), RuntimeError> {
    do_worker_stuff()?;
    Ok(())
}
```

That is the target shape:

- helper returns `Result<_, WorkerError>`
- outer layer has `RuntimeError::Worker(#[from] WorkerError)`
- the caller uses `?` instead of `map_err(|err| err.to_string())`
- `Display` still renders one human message at the boundary

Concrete repo translation examples:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("worker failed")]
    Worker(#[from] WorkerError),
}
```

```rust
fn run_worker() -> Result<(), RuntimeError> {
    do_worker_stuff()?;
    Ok(())
}
```

That same pattern is what `src/runtime/node.rs`, `src/api/worker.rs`, `src/process/planner.rs`, and `src/pginfo/state.rs` should lean on:

- source helper returns the typed source error
- outer enum wraps it with `#[from]`
- caller uses `?`
- only the final `Display` boundary renders text

## How to Untangle Smell 9

1. Find the first `String` conversion in the error path.
2. Ask whether downstream code needs to branch on the cause.
3. If yes, replace the string bucket with a typed enum variant.
4. If the source maps cleanly to one variant, add `#[from]`.
5. If the error needs context, store the context as fields, not prose.
6. If multiple distinct failures are being collapsed, split them into separate variants.
7. Once `#[from]` is in place, let `?` carry the typed source through instead of hand-writing `map_err` glue.
8. Keep only the final human-readable rendering at the boundary that talks to users, logs, or HTTP clients.
9. Re-run `make check`, then follow any remaining `String` conversions until the boundary is actually clean.

## Preferred Repo Direction

In this repo, prefer:

- typed source errors in `src/`
- typed harness errors in `tests/ha/support/`
- `String` only at the very outermost boundary when the output is genuinely human-facing
- `From` and `#[from]` for one-to-one error wrapping
- `PathBuf`, `MemberId`, `url::ParseError`, `std::io::Error`, `serde_json::Error`, and similar concrete types over free-form text
- `#[from]` on source-carrying variants whenever the mapping is direct
- `?` at the call site instead of `map_err(|err| err.to_string())` once the enum supports it

Avoid:

- `Result<_, String>` in internal code
- `Message(String)` catch-alls unless the module is already purely a presentation boundary
- converting to `String` and then converting back to structure elsewhere
- aggregating failures into `Vec<String>` if the caller can still use typed cases

This is related to smell 4, but different:

- smell 4 is about early presentation strings
- smell 9 is about early error flattening
