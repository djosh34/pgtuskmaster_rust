# Smell 4: Display Boundary, Not String Soup

This smell is about creating `String` values too early.

The preferred shape is:

1. execute the command
2. parse raw response data once with serde
3. convert once into one command output enum or struct
4. render that type directly via `Display`

Any time you use format!, use must be very skeptical and question yourself: can this stay in the same type until later?


# Example:

Why is this not Display trait? Display + write should be split up
```rust
async fn run_watch(context: &OperatorContext, options: StatusOptions) -> Result<String, CliError> {
    let mut stdout = std::io::stdout();
    let interval = Duration::from_secs(2);

    loop {
        let rendered = fetch_state_command_output(context, options.verbose)
            .await
            .and_then(|output| {
                CommandOutputDto::State {
                    output: Box::new(output),
                }
                .render(options.json)
            })?;
        if options.json {
            writeln!(stdout, "{rendered}").map_err(CliError::OutputWrite)?;
        } else {
            writeln!(stdout, "\x1B[2J\x1B[H{rendered}").map_err(CliError::OutputWrite)?;
        }
        stdout.flush().map_err(CliError::OutputFlush)?;

        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(String::new()),
            _ = tokio::time::sleep(interval) => {}
        }
    }
}

```



# Example

Why not make a roleblock type + Display?

```rust
fn render_protected_role_provision_block(spec: &ManagedRoleSpec) -> String {
    let username_literal = sql_literal(spec.username.as_str());
    let attributes = spec.identity.attributes();
    let password_literal = sql_literal(spec.password.as_str());
    format!(
        "DO $$\nBEGIN\n  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = {username_literal}) THEN\n    EXECUTE format('CREATE ROLE %I', {username_literal});\n  END IF;\n  EXECUTE format('ALTER ROLE %I WITH {attributes} PASSWORD %L', {username_literal}, {password_literal});\nEND\n$$;"
    )
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn sql_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
```