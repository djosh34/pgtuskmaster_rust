# Smell 5: One Shared Shape, One Parse, One Render

This smell is about the same concept being modeled or rendered in several near-identical ways.

The canonical example is PostgreSQL connection info.

The target shape is:

- one shared type
- one parse function
- one render function or one `Display` impl
- every caller reuses it

If the same DSN or conninfo shape is rendered manually in several places, that is boundary drift.

## Detection checklist

Look for:

- repeated manual `format!("host=... port=...")`
- repeated lists of the same connection fields
- multiple structs carrying the same connection facts with different names
- the same information represented twice inside one type

## Example A: the repo already has a canonical conninfo type

From `src/pginfo/conninfo.rs`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgConnInfo {
    pub endpoint: PgEndpoint,
    pub user: String,
    pub dbname: String,
    pub application_name: Option<String>,
    pub connect_timeout_s: Option<u32>,
    pub ssl_mode: PgSslMode,
    pub ssl_root_cert: Option<PathBuf>,
    pub options: Option<String>,
    pub tls: PgClientTls,
}

impl fmt::Display for PgConnInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(render_pg_conninfo(self).as_str())
    }
}

impl FromStr for PgConnInfo {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let entries = parse_conninfo_entries(input)?;
        // ...
    }
}
```

This is already close to the desired shape:

- one shared struct
- parse once
- render once

That means any extra ad-hoc DSN logic elsewhere should be challenged first.

## Example B: the canonical type still duplicates information internally

From the same file:

```rust
pub struct PgConnInfo {
    // ...
    pub ssl_mode: PgSslMode,
    pub ssl_root_cert: Option<PathBuf>,
    pub options: Option<String>,
    pub tls: PgClientTls,
}

pub struct PgClientTls {
    pub mode: PgSslMode,
    pub root_cert: Option<PathBuf>,
    pub client_cert: Option<PathBuf>,
    pub client_key: Option<SecretSource>,
}
```

This is a boundary smell inside the type itself:

- `ssl_mode` duplicates `tls.mode`
- `ssl_root_cert` duplicates `tls.root_cert`
- the type allows hybrid shapes because plaintext and TLS facts live together

This is exactly where a flatter enum can win.

A better direction is the shape you described:

- `PgConnSocket`
- `PgConnNetwork`
- `PgConnNetworkTls`

In Rust terms that likely means one enum where each variant carries only the facts that make sense for that connection mode.

The point is:

- socket connections should not need extra config to be usable
- non-TLS network connections should not carry TLS-only fields
- TLS network connections should always carry the TLS sub-struct they require

## Example C: ad-hoc DSN rendering still appears elsewhere

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
    // ...
}
```

From `src/postgres_managed.rs` tests:

```rust
let primary_dsn = format!(
    "host=127.0.0.1 port={} user=postgres dbname=postgres",
    primary_port
);
```

```rust
let replica_dsn = format!(
    "host=127.0.0.1 port={} user=postgres dbname=postgres",
    replica_port
);
```

These are good smell examples because a canonical `PgConnInfo` already exists.

That means:

- the project already knows the concept deserves a shared type
- these manual strings are now duplicate local mini-languages

## Example D: repeated construction of the same conninfo facts

From `src/process/source.rs`:

```rust
fn remote_conninfo(
    member: &ClusterMemberView,
    role: &MandatoryPostgresRoleCredential,
    runtime: &ProcessRuntimePlan,
) -> PgConnInfo {
    PgConnInfo {
        endpoint: member.postgres_target().clone(),
        user: role.username.as_str().to_owned(),
        dbname: runtime.replica_access.dbname.clone(),
        application_name: None,
        connect_timeout_s: Some(runtime.replica_access.connect_timeout_s),
        ssl_mode: runtime.replica_access.ssl_mode,
        ssl_root_cert: runtime.replica_access.ssl_root_cert.clone(),
        options: None,
        tls: PgClientTls {
            mode: runtime.replica_access.ssl_mode,
            root_cert: runtime.replica_access.ssl_root_cert.clone(),
            client_cert: None,
            client_key: None,
        },
    }
}
```

Even when the shared type is used, duplicated field setting inside it is still a smell.

Ask:

- can a constructor own this invariant once?
- can the type itself remove duplicate fields?
- can this become one enum variant instead of a struct with optional and repeated fields?
