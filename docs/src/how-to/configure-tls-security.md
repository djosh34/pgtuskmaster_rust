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
- API callers are authenticated with the right bearer token
- optional client certificate verification is enabled only when you have the required CA material

## Decide What You Are Protecting

The runtime exposes two very different surfaces:

- `api.security.tls` protects the HTTP control plane
- `postgres.tls` protects the PostgreSQL server side

Think about them separately during rollout:

- API TLS protects operator traffic such as `/ha/state`, `/switchover`, and debug endpoints.
- PostgreSQL TLS protects database client and replication traffic.

## Harden The API Surface

Start from an API security block that enables TLS and authorization together:

```toml
[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "required", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } } }, auth = { type = "role_tokens", read_token = "read-secret", admin_token = "admin-secret" } }
```

The role split matters operationally:

- `read_token` is enough for observation endpoints such as `/ha/state`
- `admin_token` is required for write operations such as `POST /switchover` and `DELETE /ha/switchover`

If `read_token` is absent but `admin_token` is present, the admin token still authorizes reads.

## Decide Whether To Require Client Certificates

The TLS schema supports optional client certificate verification:

```toml
security = { tls = { mode = "required", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } }, client_auth = { client_ca = { path = "/etc/pgtuskmaster/tls/client-ca.pem" }, require_client_cert = true } }, auth = { type = "role_tokens", read_token = "read-secret", admin_token = "admin-secret" } }
```

Use `require_client_cert = true` only when every intended client has a certificate chain rooted in the configured CA bundle.

Use `require_client_cert = false` if you want the server to validate presented client certificates without making them mandatory for every caller.

## Harden PostgreSQL Transport

For PostgreSQL, the runtime schema supports the same TLS shape:

```toml
[postgres]
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "require" }
rewind_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "require" }
tls = { mode = "required", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/postgres-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/postgres-key.pem" } } }
```

Security-sensitive points to verify:

- the runtime can read the configured key material
- connection identities use a TLS-capable `ssl_mode`
- replication and rewind paths are updated alongside ordinary SQL clients

## Verify Authorization After Enabling TLS

Query a read endpoint with the read token:

```bash
curl --fail --silent \
  -H "Authorization: Bearer $PGTUSKMASTER_READ_TOKEN" \
  https://127.0.0.1:8080/ha/state
```

Try an admin operation with the admin token:

```bash
pgtuskmasterctl \
  --base-url https://127.0.0.1:8080 \
  --admin-token "$PGTUSKMASTER_ADMIN_TOKEN" \
  ha switchover request \
  --requested-by node-a
```

That verifies both layers at once:

- the transport accepts HTTPS
- the API enforces admin-vs-read privileges correctly

## Keep The Layers Mentally Separate

When a request fails, ask which layer failed:

- TLS handshake problem: certs, keys, CA bundle, or TLS mode
- HTTP `401 Unauthorized`: missing or invalid bearer token
- HTTP `403 Forbidden`: valid token for the wrong role
- application-level `503` or other runtime errors: the request reached the application, but the underlying operation failed

This distinction matters because bearer-token failures are not evidence that TLS is misconfigured, and TLS handshake failures are not evidence that role-token auth is broken.

## Troubleshooting

### API starts, but every request is rejected

Check the `Authorization: Bearer ...` header first. If role tokens are enabled, the API rejects unauthenticated requests even when TLS itself is working.

### HTTPS works, but admin operations fail

Confirm you are using the admin token, not a read-only token. Read-role access is not enough for switchover endpoints.

### Enabling client certificate verification locks out clients

Check the configured `client_ca` bundle and whether the callers actually present certificates rooted in that CA. If you are still migrating clients, use optional verification instead of mandatory client certs.

### PostgreSQL traffic still appears to be plaintext

Check whether the deployment is still on `mode = "optional"` or whether clients are still connecting with a non-TLS `sslmode`.
