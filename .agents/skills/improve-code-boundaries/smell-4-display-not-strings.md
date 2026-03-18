# Smell 4: Display Boundary, Not String Soup

This smell is about creating `String` values too early.

The preferred shape is:

1. execute the command
2. parse raw response data once with serde
3. convert once into one command output enum or struct
4. render that type directly via `Display`

Do not build presentation strings in the middle of the pipeline if the data can stay typed a little longer.

## Detection checklist

Look for these signals:

- helpers named `render_*` that return `String` long before the final output boundary
- DTOs that store presentation-ready strings instead of typed values
- CLI code that repeatedly calls `format!`, `.to_string()`, `.join()`, or `push_str()` to assemble output
- one top-level `Display` impl that still delegates to many lower string factories

## Example A: the repo already has the right top-level boundary

From `src/cli/output.rs`:

```rust
pub fn render_command_output(value: &CommandOutputDto, json: bool) -> Result<String, CliError> {
    if json {
        serde_json::to_string_pretty(value)
            .map_err(|err| CliError::Output(format!("json encode failed: {err}")))
    } else {
        Ok(value.to_string())
    }
}
```

That is the correct top-level shape:

- JSON mode serializes the typed output once
- text mode calls `Display`

From `src/command/mod.rs`:

```rust
impl fmt::Display for CommandOutputDto {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = match self {
            Self::State { output } => render_state_command_text(output),
            Self::Primary { output } | Self::Replicas { output } => {
                render_connection_command_text(output)
            }
            Self::Switchover { output } => format!("accepted={}", output.accepted),
            Self::ReloadCertificates { output } => match serde_json::to_string_pretty(output) {
                Ok(json) => json,
                Err(_) => "failed to encode reload certificates response".to_string(),
            },
        };
        formatter.write_str(rendered.as_str())
    }
}
```

This is partially good and partially smelly.

Good:

- one output enum owns rendering

Smelly:

- the `Display` impl still depends on several lower helpers that already return `String`
- rendering details are spread across string factories instead of smaller `Display` impls

## Example B: connection output is rendered by manual string assembly

From `src/command/mod.rs`:

```rust
pub fn materialize_connection_dsn(
    target: &StateDerivedConnectionTargetDto,
    local: &LocalConnectionMaterialization,
) -> String {
    let base_fields = [
        ("host", target.postgres_host.clone()),
        ("port", target.postgres_port.to_string()),
        ("user", "postgres".to_string()),
        ("dbname", "postgres".to_string()),
    ];

    base_fields
        .into_iter()
        .chain(tls_fields)
        .map(|(key, value)| format!("{key}={}", render_conninfo_value(value.as_str())))
        .collect::<Vec<_>>()
        .join(" ")
}
```

This is string soup because:

- typed connection data is flattened into string pairs early
- the code is manually assembling one textual representation in a helper
- it is no longer obvious whether this is a shared connection concept or just a one-off rendering trick

This should prefer one typed connection-output struct with `Display`, or reuse a canonical shared conninfo type directly.

## Example C: state text rendering is one large string builder

From `src/command/mod.rs`:

```rust
fn render_state_command_text(output: &StateCommandOutputDto) -> String {
    let header_lines = [
        format!(
            "cluster: {}  health: {}",
            projection.cluster_name,
            health_label(projection.health)
        ),
        format!("queried via: {}", projection.queried_via.api_url),
    ];
    let warning_lines = projection
        .warnings
        .iter()
        .map(|warning| format!("warning: {}", warning.message));
```

This is a sign to ask:

- which parts are real output semantics and belong in typed output values?
- which parts are pure final formatting and belong in `Display` for smaller leaf types?

The problem is not `format!` itself. The problem is using it as the primary data model.

## How to untangle smell 4

1. Find the earliest point where a `String` is created for presentation.
2. Comment out that helper if possible and run `make check`.
3. Ask what typed value that string was representing.
4. Introduce or reuse a typed output enum or struct for that concept.
5. Move the actual string formatting to `Display` on that type.
6. Let the top-level command output own rendering instead of lower helper layers.

## Preferred CLI recipe for this repo

- client gets JSON
- serde parses once
- command layer builds one `CommandOutputDto`
- nested output structs keep typed fields for as long as possible
- only `Display` turns them into text

If you find several text-producing helpers below that boundary, flatten them.

