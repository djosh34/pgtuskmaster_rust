# Configure TLS Security

This guide focuses on the security posture around TLS-enabled deployments. Use it after you understand the basic enablement steps in [Configure TLS](configure-tls.md).

The runtime has two separate security layers:

- transport security through TLS
- request authorization through API role tokens

They are independent. You can enable one without the other, but production deployments usually need both.

## Goal

Harden a node so that:

- the API is served over TLS
- PostgreSQL traffic can be protected with TLS
- `pgtm` authenticates with the right role token
- optional client certificate verification is enabled only when you have the required CA material

## Decide what you are protecting

The runtime exposes two very different surfaces:

- `api.security.tls` protects the HTTP control plane
- `postgres.tls` protects the PostgreSQL server side

Think about them separately during rollout:

- API TLS protects operator traffic such as status, switchover, and debug reads
- PostgreSQL TLS protects database client and replication traffic

## Harden the API surface

Start from an API security block that enables TLS and authorization together:

```toml
[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "required", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } } }, auth = { type = "role_tokens", read_token = "read-secret", admin_token = "admin-secret" } }
```

The role split matters operationally:

- `read_token` is enough for observation commands such as `status` and `debug verbose`
- `admin_token` is required for write operations such as `switchover request` and `switchover clear`

If `read_token` is absent but `admin_token` is present, the admin token still authorizes reads.

## Add the matching `pgtm` client context

Keep the operator client configuration in the same shared config:

```toml
[pgtm]
api_url = "https://db-a.example.com:8443"

[pgtm.api_client]
ca_cert = { path = "/etc/pgtm/api-ca.pem" }
client_cert = { path = "/etc/pgtm/operator.crt" }
client_key = { path = "/etc/pgtm/operator.key" }
```

Use `client_cert` and `client_key` only when the API requires client certificates. Leave them out when TLS is server-authenticated only.

## Decide whether to require client certificates

The TLS schema supports optional client certificate verification:

```toml
security = { tls = { mode = "required", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } }, client_auth = { client_ca = { path = "/etc/pgtuskmaster/tls/client-ca.pem" }, require_client_cert = true } }, auth = { type = "role_tokens", read_token = "read-secret", admin_token = "admin-secret" } }
```

Use `require_client_cert = true` only when every intended client has a certificate chain rooted in the configured CA bundle.

Use `require_client_cert = false` if you want the server to validate presented client certificates without making them mandatory for every caller.

## Harden PostgreSQL transport

For PostgreSQL, the runtime schema supports the same TLS shape:

```toml
[postgres]
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "require" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "verify-full", ca_cert = { path = "/etc/pgtuskmaster/tls/postgres-ca.pem" } }
tls = { mode = "required", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/postgres-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/postgres-key.pem" } } }
```

Security-sensitive points to verify:

- the runtime can read the configured key material
- connection identities use a TLS-capable `ssl_mode`
- every `verify_ca` or `verify_full` identity also carries a readable `ca_cert` path
- replication and rewind paths are updated alongside ordinary SQL clients

If operators need TLS-expanded DSN output from `pgtm`, also configure `[pgtm.postgres_client]`.

## Verify authorization after enabling TLS

Query a read operation through the CLI:

```bash
pgtm -c config.toml status
```

Then try an admin operation:

```bash
pgtm -c config.toml switchover request
```

That verifies both layers at once:

- the transport accepts HTTPS
- the API enforces admin-vs-read privileges correctly

## Keep the layers mentally separate

When a request fails, ask which layer failed:

- TLS handshake problem: certs, keys, CA bundle, or TLS mode
- HTTP `401 Unauthorized`: missing or invalid configured role token
- HTTP `403 Forbidden`: valid token for the wrong role
- application-level `503` or other runtime errors: the request reached the application, but the underlying operation failed

This distinction matters because role-token failures are not evidence that TLS is misconfigured, and TLS handshake failures are not evidence that auth is broken.

## Troubleshooting

### API starts, but every CLI request is rejected

Check the configured `read_token` and `admin_token` first. If role tokens are enabled, `pgtm` still needs valid credentials even when TLS itself is working.

### HTTPS works, but admin operations fail

Confirm you are using the admin token, not a read-only token. Read-role access is not enough for switchover commands.

### Enabling client certificate verification locks out clients

Check the configured `client_ca` bundle and whether the callers actually present certificates rooted in that CA. If you are still migrating clients, use optional verification instead of mandatory client certs.

### PostgreSQL traffic still appears to be plaintext

Check whether the deployment is still on `mode = "optional"` or whether clients are still connecting with a non-TLS `sslmode`.
