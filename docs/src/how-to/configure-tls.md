# Configure TLS

This guide shows how to turn the runtime's TLS knobs into an actual deployment procedure while keeping `pgtm` truthful as the normal operator client.

## Goal

Enable TLS for one or both of these server surfaces:

- the `pgtuskmaster` HTTP API
- the managed PostgreSQL server

## What the schema supports

The runtime schema now models the two surfaces differently:

```toml
[postgres]
tls = { mode = "disabled" }
tls = { mode = "enabled", identity = { cert_chain = { path = "/path/to/chain.pem" }, private_key = { path = "/path/to/key.pem" } } }
tls = { mode = "enabled", identity = { cert_chain = { path = "/path/to/chain.pem" }, private_key = { path = "/path/to/key.pem" } }, client_auth = { client_ca = { path = "/path/to/ca.pem" }, client_certificate = "optional" } }

[api]
security = { transport = { transport = "http" }, auth = { type = "disabled" } }
security = { transport = { transport = "https", tls = { identity = { cert_chain = { path = "/path/to/chain.pem" }, private_key = { path = "/path/to/key.pem" } } } }, auth = { type = "disabled" } }
```

For PostgreSQL `mode = "enabled"`, the runtime expects:

- `identity.cert_chain`
- `identity.private_key`

If PostgreSQL `client_auth` is configured, it also expects:

- `client_auth.client_ca`
- `client_auth.client_certificate`

For the API, `transport = "https"` requires the `tls.identity` block. If API `client_auth` is configured, it expects `client_ca` and the config-file boolean `require_client_cert`.

If those values are missing, the TLS loader rejects the config during startup.

## Choose the rollout shape

There is no mixed plaintext-plus-TLS server mode anymore.

Keep the current surface on `transport = "http"` or `postgres.tls.mode = "disabled"` until clients are ready, then cut over to `transport = "https"` or `postgres.tls.mode = "enabled"` during a controlled restart.

## Enable API TLS

Update the `api.security` block:

```toml
[api]
listen_addr = "0.0.0.0:8080"
security = { transport = { transport = "https", tls = { identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } } } }, auth = { type = "disabled" } }
```

If you want client certificate verification, add `client_auth`:

```toml
[api]
listen_addr = "0.0.0.0:8080"
security = { transport = { transport = "https", tls = { identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } }, client_auth = { client_ca = { path = "/etc/pgtuskmaster/tls/client-ca.pem" }, require_client_cert = true } } }, auth = { type = "disabled" } }
```

## Add operator client settings for `pgtm`

If the daemon binds an unspecified address such as `0.0.0.0:8080`, add an operator-facing `[pgtm]` section so the CLI resolves the reachable HTTPS URL and any client TLS material correctly:

```toml
[pgtm]
api_url = "https://db-a.example.com:8443"

[pgtm.api_client]
ca_cert = { path = "/etc/pgtm/api-ca.pem" }
client_cert = { path = "/etc/pgtm/operator.crt" }
client_key = { path = "/etc/pgtm/operator.key" }
```

Use `client_cert` and `client_key` only when the API server requires client certificates. If the API uses bearer tokens, keep those in `api.security.auth` so `pgtm` can resolve them from the same config.

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
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "verify-full", ca_cert = { path = "/etc/pgtuskmaster/tls/postgres-ca.pem" } }
tls = { mode = "enabled", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/postgres-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/postgres-key.pem" } } }
pg_hba = { source = { path = "/etc/pgtuskmaster/pg_hba.conf" } }
pg_ident = { source = { path = "/etc/pgtuskmaster/pg_ident.conf" } }
```

Two details matter here:

- `local_conn_identity.ssl_mode` and `rewind_conn_identity.ssl_mode` must match the fact that the server now expects TLS-capable connections
- `verify_ca` and `verify_full` also require an explicit `ca_cert` path in the matching connection identity so runtime-managed conninfo can render `sslrootcert=...`
- if `postgres.tls.mode` stays `disabled`, the runtime rejects TLS-required client modes such as `require`, `verify_ca`, or `verify_full`

If you want `pgtm primary --tls` and `pgtm replicas --tls` to emit PostgreSQL client certificate paths, also configure:

```toml
[pgtm.postgres_client]
ca_cert = { path = "/etc/pgtm/postgres-ca.pem" }
client_cert = { path = "/etc/pgtm/postgres.crt" }
client_key = { path = "/etc/pgtm/postgres.key" }
```

## Put the certificate material in place

The repository does not currently ship a dedicated certificate-generation workflow or checked-in certificate assets. Treat certificate generation and secret distribution as deployment-specific work.

Before restarting nodes, make sure every configured path exists on disk and is readable by the runtime:

- API server cert chain and private key
- PostgreSQL server cert chain and private key
- optional client CA bundle for mutual TLS
- any `pgtm` client CA/cert/key material you configured

## Roll out the config

Apply the updated runtime config and TLS files to each node, then restart the runtime in a controlled sequence.

For a staged rollout:

1. Deploy cert/key files to every node.
2. Update client trust material and TLS settings.
3. Change the target surface to `transport = "https"` or `mode = "enabled"`.
4. Restart the node.
5. Verify clients reconnect with TLS.

For an all-at-once rollout:

1. Stop client traffic or schedule a maintenance window.
2. Deploy cert/key files and config changes.
3. Restart the nodes.
4. Verify clients reconnect with TLS.

## Verify API TLS

Once API TLS is enabled, verify it through `pgtm`:

```bash
pgtm -c config.toml status
```

That confirms the CLI can reach the HTTPS API with the configured CA bundle, optional client certificates, and any configured auth tokens.

## Verify PostgreSQL TLS

Once PostgreSQL TLS is enabled, resolve the current primary with TLS-expanded DSN output:

```bash
pgtm -c config.toml primary --tls
```

Then verify a real PostgreSQL client can connect with those settings:

```bash
psql "$(pgtm -c config.toml primary --tls)"
```

## Troubleshooting

### Startup fails immediately after enabling TLS

Check the configured paths first. The TLS loader fails startup if `transport = "https"` or `mode = "enabled"` is set but the required identity files are missing, unreadable, or not valid PEM.

### `pgtm` cannot reach the API after the change

Check:

- `[pgtm].api_url` points at the reachable HTTPS URL
- `[pgtm.api_client]` has the right CA bundle and, when needed, client cert/key
- the configured auth tokens still match the server

### PostgreSQL clients fail after the change

Confirm that:

- the server-side TLS files exist on the node
- client connection strings now request TLS
- `local_conn_identity` and `rewind_conn_identity` use an `ssl_mode` compatible with the server mode

### Mutual TLS rejects clients unexpectedly

Check `api.security.transport.tls.client_auth.require_client_cert` and the configured `client_ca` path. The runtime uses the supplied CA bundle to build the client certificate verifier.
