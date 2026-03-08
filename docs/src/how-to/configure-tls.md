# Configure TLS

This guide shows how to turn the runtime's TLS knobs into an actual deployment procedure.

The important starting point is that the shipped Docker cluster configs currently keep both `postgres.tls.mode` and `api.security.tls.mode` set to `disabled`. This guide therefore describes how to adapt a deployment for TLS; it is not a claim that the checked-in cluster example already runs with TLS enabled.

## Goal

Enable TLS for one or both of these server surfaces:

- the pgtuskmaster HTTP API
- the managed PostgreSQL server

## What The Schema Supports

The runtime schema defines the same TLS shape for both servers:

```toml
tls = { mode = "disabled" }
tls = { mode = "optional", identity = { cert_chain = { path = "/path/to/chain.pem" }, private_key = { path = "/path/to/key.pem" } } }
tls = { mode = "required", identity = { cert_chain = { path = "/path/to/chain.pem" }, private_key = { path = "/path/to/key.pem" } } }
```

When `mode` is `optional` or `required`, the runtime expects:

- `identity.cert_chain`
- `identity.private_key`

If `client_auth` is configured, it also expects:

- `client_auth.client_ca`
- `client_auth.require_client_cert`

If those values are missing, the TLS loader rejects the config during startup.

## Choose The Rollout Shape

Use `mode = "optional"` when you need a staged rollout and still have clients that connect without TLS.

Use `mode = "required"` when all clients are ready to connect with TLS and you want the server surface to reject plaintext traffic.

## Enable API TLS

Update the `api.security` block:

```toml
[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "required", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } } }, auth = { type = "disabled" } }
```

If you want client certificate verification, add `client_auth`:

```toml
[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "required", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } }, client_auth = { client_ca = { path = "/etc/pgtuskmaster/tls/client-ca.pem" }, require_client_cert = true } }, auth = { type = "disabled" } }
```

## Enable PostgreSQL TLS

Update the PostgreSQL block:

```toml
[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "node-a"
listen_port = 5432
socket_dir = "/var/lib/pgtuskmaster/socket"
log_file = "/var/log/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "require" }
rewind_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "require" }
tls = { mode = "required", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/postgres-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/postgres-key.pem" } } }
pg_hba = { source = { path = "/etc/pgtuskmaster/pg_hba.conf" } }
pg_ident = { source = { path = "/etc/pgtuskmaster/pg_ident.conf" } }
```

Two details matter here:

- `local_conn_identity.ssl_mode` and `rewind_conn_identity.ssl_mode` must match the fact that the server now expects TLS-capable connections.
- If `postgres.tls.mode` stays `disabled`, the runtime rejects TLS-required client modes such as `require`, `verify_ca`, or `verify_full`.

## Put The Certificate Material In Place

The repository does not currently ship a dedicated certificate-generation workflow or checked-in certificate assets. Treat certificate generation and secret distribution as deployment-specific work.

Before restarting nodes, make sure every configured path exists on disk and is readable by the runtime:

- API server cert chain and private key
- PostgreSQL server cert chain and private key
- optional client CA bundle for mutual TLS

## Roll Out The Config

Apply the updated runtime config and TLS files to each node, then restart the runtime in a controlled sequence.

For a staged rollout:

1. Deploy cert/key files to every node.
2. Change the target surface to `mode = "optional"`.
3. Restart the node.
4. Update clients to use TLS.
5. Once all clients are migrated, change `mode = "required"`.

For an all-at-once rollout:

1. Stop client traffic or schedule a maintenance window.
2. Deploy cert/key files and config changes.
3. Restart the nodes.
4. Verify clients reconnect with TLS.

## Verify API TLS

Once API TLS is enabled, query an endpoint over HTTPS:

```bash
curl --fail --silent https://127.0.0.1:8080/ha/state
```

If the certificate is not yet trusted by your local client environment, add the appropriate CA configuration for your environment rather than assuming insecure verification is acceptable as a steady state.

If API bearer tokens are enabled, include the required header as usual. TLS and bearer-token auth are separate controls.

## Verify PostgreSQL TLS

Once PostgreSQL TLS is enabled, connect with a client mode that requires TLS:

```bash
psql "host=node-a port=5432 user=postgres dbname=postgres sslmode=require"
```

The exact client certificate flags depend on your deployment and are not prescribed by the repository, but the important validation point is that plaintext connections should no longer be the only successful path once you have switched clients over.

## Troubleshooting

### Startup fails immediately after enabling TLS

Check the configured paths first. The TLS loader fails startup if `mode` requires TLS but `identity` is missing or unreadable, or if the PEM files cannot be parsed.

### API still serves plaintext traffic

Check whether you left `api.security.tls.mode = "optional"`. Optional mode is useful for migration, but it is not an enforcement mode.

### PostgreSQL clients fail after the change

Confirm that:

- the server-side TLS files exist on the node
- client connection strings now request TLS
- `local_conn_identity` and `rewind_conn_identity` use an `ssl_mode` compatible with the server mode

### Mutual TLS rejects clients unexpectedly

Check `client_auth.require_client_cert` and the configured `client_ca` path. The runtime uses the supplied CA bundle to build the client certificate verifier.
