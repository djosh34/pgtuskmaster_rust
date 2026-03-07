# Prerequisites

Before the first launch, confirm the node has everything it needs locally. Most failed first runs come from missing binaries, unreadable secrets, wrong directory ownership, or an API bind address that does not match how you intend to operate the node.

Required on the node host:

- PostgreSQL 16 binaries at the absolute paths you will place in `process.binaries`:
  `postgres`, `pg_ctl`, `pg_rewind`, `initdb`, `pg_basebackup`, and `psql`
- Reachability to at least one etcd endpoint in the cluster scope you plan to use
- Writable local paths for `postgres.data_dir`, `postgres.socket_dir`, and `postgres.log_file`
- A `config_version = "v2"` runtime config file

Confirm these before you start:

- The runtime account can read every referenced secret or certificate file.
- The PostgreSQL data directory is empty if this will be an initial bootstrap.
- The socket directory path is short enough for Unix socket limits.
- The chosen `api.listen_addr` is deliberate. `127.0.0.1:8080` is the default if you omit it.
- If you plan to expose the API off-host, you already have TLS material and API tokens ready instead of falling back to an open HTTP listener.

If any of those are uncertain, fix them before launching. The startup path is much easier to reason about when the node is not simultaneously fighting filesystem, auth, and network mistakes.
