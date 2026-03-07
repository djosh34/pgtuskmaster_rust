# Configuration Guide

This guide describes the current `config_version = "v2"` runtime schema. The parser is strict on purpose: it refuses missing security-sensitive fields, unreadable secret paths, and incomplete runtime surfaces instead of inventing defaults that would hide deployment mistakes.

The fastest way to understand a working configuration is to read the checked-in container examples under `docker/configs/**`. Those files are real runtime configs, not illustrative pseudocode. Use them as the reference point before you translate the runtime into another deployment system.

## Container-first baseline

This is the shape the Compose stacks use today:

```toml
config_version = "v2"

[cluster]
name = "docker-cluster"
member_id = "node-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "node-a"
listen_port = 5432
socket_dir = "/var/lib/pgtuskmaster/socket"
log_file = "/var/log/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
rewind_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
tls = { mode = "disabled" }
roles = {
  superuser = { username = "postgres", auth = { type = "password", password = { path = "/run/secrets/postgres-superuser-password" } } },
  replicator = { username = "postgres", auth = { type = "password", password = { path = "/run/secrets/replicator-password" } } },
  rewinder = { username = "postgres", auth = { type = "password", password = { path = "/run/secrets/rewinder-password" } } },
}
pg_hba = { source = { path = "/etc/pgtuskmaster/pg_hba.conf" } }
pg_ident = { source = { path = "/etc/pgtuskmaster/pg_ident.conf" } }

[dcs]
endpoints = ["http://etcd:2379"]
scope = "docker-cluster"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = {
  postgres = "/usr/lib/postgresql/16/bin/postgres",
  pg_ctl = "/usr/lib/postgresql/16/bin/pg_ctl",
  pg_rewind = "/usr/lib/postgresql/16/bin/pg_rewind",
  initdb = "/usr/lib/postgresql/16/bin/initdb",
  pg_basebackup = "/usr/lib/postgresql/16/bin/pg_basebackup",
  psql = "/usr/lib/postgresql/16/bin/psql",
}

[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }

[debug]
enabled = true
```

That baseline is intentionally lab-oriented. It keeps file-backed passwords visible, shares one API port for operational and debug routes, and disables TLS and token auth because the quick-start path is scoped as a local-only environment rather than pretending to be production-ready.

## Hardened operator baseline

For a real operator-facing deployment, keep the same secret-file discipline but harden the API surface:

```toml
[api]
listen_addr = "10.0.0.41:8080"
security = {
  tls = {
    mode = "required",
    identity = {
      cert_chain = { path = "/run/secrets/api-server.crt" },
      private_key = { path = "/run/secrets/api-server.key" }
    }
  },
  auth = { type = "role_tokens", read_token = "REPLACE_WITH_READ_TOKEN", admin_token = "REPLACE_WITH_ADMIN_TOKEN" }
}

[debug]
enabled = false
```

Two caveats still matter:

- `api.security.auth.role_tokens.*` are plain strings in the schema today, so render the final TOML from a protected deployment path instead of committing real tokens
- `pg_hba` and `pg_ident` remain operator-owned surfaces; the runtime consumes them, but it does not invent correct policy for you

## Read the schema by behavior, not only by section name

### `config_version`

`v2` is the only supported version. Treat that as a compatibility guard, not as decorative metadata. If a config does not claim the expected schema version, the runtime is telling you it would rather stop than guess how to interpret security or process settings.

### `[cluster]`

`name` labels the cluster for operator-facing surfaces, while `member_id` is the stable local identity written into DCS and API responses. A mismatched member ID is not a cosmetic typo. It changes how the node represents itself in the cluster and can make diagnostics look incoherent across nodes.

### `[postgres]`

This block controls the managed PostgreSQL contract:

- `data_dir`, `socket_dir`, and `log_file` define the local filesystem layout the runtime will expect to manage
- `listen_host` and `listen_port` define how the node advertises PostgreSQL reachability
- `local_conn_identity` and `rewind_conn_identity` shape how internal control and rewind paths connect
- `roles.*` define the file-backed authentication material the process worker will need for startup, replication, and rewind paths
- `pg_hba` and `pg_ident` sources define the managed auth files the runtime materializes into the PostgreSQL startup surface

The practical lesson is that most PostgreSQL failures under `pgtuskmaster` are not random. They usually reflect one of three classes: unreadable secret material, a policy mismatch in `pg_hba` or `pg_ident`, or a wrong local path/identity assumption that later shows up during base backup or rewind.

### `[dcs]`

`endpoints` must point at reachable etcd listeners, and `scope` must be identical across every node in the same cluster. If scope diverges, nodes are not "slightly misconfigured." They are effectively participating in different coordination universes.

### `[ha]`

The HA timing block defines how quickly the node reevaluates state and how long leader ownership may remain valid:

- `loop_interval_ms` controls reconciliation cadence
- `lease_ttl_ms` must stay greater than `loop_interval_ms`

The tradeoff is direct. Smaller intervals can make the system more reactive, but they also tighten the tolerance for transient latency and coordination noise. If you make HA timing aggressive, expect noisy environments to look more unstable rather than magically more available.

### `[process]`

Every PostgreSQL binary path must be absolute. The process worker refuses relative or missing binaries because job dispatch is supposed to be deterministic. If this block is wrong, startup, base backup, rewind, and local control operations will all fail downstream in different-looking ways.

### `[api]`

There is exactly one API listener. When `debug.enabled = true`, `/debug/*` rides the same listener. Do not design a second debug port around the docs; that would contradict the actual runtime contract and the quick-start topology.

### `[debug]`

Enable debug only when you actively need the extra routes. It is useful in labs and during focused diagnosis, but in a hardened deployment it expands the observation surface you must secure and interpret.

## Configuration groups that most affect behavior

### Cluster identity and DCS scope

These values decide how the node names itself and where it writes coordination records. A wrong scope or member ID will produce confusing symptoms: the node may look internally healthy while every peer seems to "ignore" it because they are not reading the same keys.

### PostgreSQL wiring and local filesystem layout

If `data_dir`, `socket_dir`, or log paths do not line up with the container or host filesystem, startup planning can succeed while actual process execution fails. This is why the docs keep pointing back to the checked-in configs: they show the complete runtime contract, not just isolated snippets.

### API exposure and security

The `api` block determines both reachability and who is allowed to do what. A node that returns `401` or `403` is not necessarily unhealthy. It may simply be doing exactly what its configured role-token model says. Operators should distinguish security mismatch from cluster-state failure early in diagnosis.

### Debug posture

Debug routes are convenient because `/debug/verbose` exposes the richer snapshot stream and timeline. They are also a deliberate surface choice. Leaving them enabled outside controlled environments increases what an attacker or over-curious client can inspect.

## Common failure signatures

| Symptom | Likely cause | First check |
| --- | --- | --- |
| startup fails before the API binds | unreadable secret file, missing required `v2` block, or invalid path wiring | parser error, file permissions, absolute path correctness |
| `docker compose config` fails | `.env.docker` points at missing secret files or malformed image/port values | `.env.docker` and referenced file paths |
| `/debug/verbose` returns `404` unexpectedly | `debug.enabled = false` or the wrong config was deployed | the `[debug]` block and actual mounted runtime file |
| replica bootstrap fails | replication credentials or `pg_hba` policy do not allow the expected source connection | `postgres.roles.replicator`, `local_conn_identity`, and HBA rules |
| rewind jobs fail | rewinder credentials, privileges, or source reachability are wrong | `postgres.rewind_conn_identity`, `postgres.roles.rewinder`, and leader endpoint visibility |
| API returns `401` or `403` | auth tokens or role split do not match the command | `api.security.auth` plus the token the client used |

Treat these as behavior clues rather than as isolated errors. They tell you which configuration group is shaping the runtime's next safe move.
