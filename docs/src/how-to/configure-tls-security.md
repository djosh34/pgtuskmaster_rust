# Configure TLS Security

This guide focuses on the security posture of a TLS-enabled deployment after you understand the basic mechanics from [Configure TLS](configure-tls.md).

## Goal

Harden a node so that:

- the API is served over TLS
- API requests use the correct role token
- optional API client certificate verification is configured deliberately
- PostgreSQL transport is protected with TLS where required

## Keep the layers separate

The runtime has two independent security layers:

- transport security through TLS
- request authorization through API role tokens

The TLS handshake can succeed while auth still rejects a request, and auth can be correct while TLS is still misconfigured.

## Harden the API surface

Start from an API block that enables TLS and authorization together:

```toml
[api]
listen_addr = "0.0.0.0:8443"
transport = { transport = "https", tls = { identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } } } }
auth = { type = "role_tokens", read_token = { content = "read-secret" }, admin_token = { content = "admin-secret" } }
```

The role split matters operationally:

- `read_token` is enough for observation commands such as `status`
- `admin_token` is required for write operations such as switchover commands

## Add the matching `pgtm` client context

Keep the operator client settings in the same runtime config:

```toml
[pgtm.api]
base_url = "https://db-a.example.com:8443"
auth = { type = "role_tokens", read_token = { content = "read-secret" }, admin_token = { content = "admin-secret" } }
tls = { ca_cert = { path = "/etc/pgtm/api-ca.pem" } }
```

If the API requires client certificates:

```toml
[pgtm.api]
tls = { ca_cert = { path = "/etc/pgtm/api-ca.pem" }, identity = { cert = { path = "/etc/pgtm/operator.crt" }, key = { path = "/etc/pgtm/operator.key" } } }
```

## Decide whether to require API client certificates

The API TLS schema supports:

- no client certificate verification
- optional verification
- required verification

Example with required client certs and an allow-list:

```toml
[api]
transport = { transport = "https", tls = { identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } }, client_auth = { client_certificate = "required", client_ca = { path = "/etc/pgtuskmaster/tls/client-ca.pem" }, allowed_common_names = ["operator-a"] } } }
auth = { type = "role_tokens", read_token = { content = "read-secret" }, admin_token = { content = "admin-secret" } }
```

Use `client_certificate = "optional"` when you are still migrating clients and want validation for presented certs without blocking callers that do not present one yet.

## Harden PostgreSQL transport

For PostgreSQL, enable server TLS and tighten rewind verification as needed:

```toml
[postgres]
tls = { mode = "enabled", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/postgres-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/postgres-key.pem" } } }

[postgres.rewind.transport]
ssl_mode = "verify_full"
ca_cert = { path = "/etc/pgtuskmaster/tls/postgres-ca.pem" }
```

If operators need TLS-expanded DSN output from `pgtm`, configure:

```toml
[pgtm.postgres.tls]
ca_cert = { path = "/etc/pgtm/postgres-ca.pem" }
identity = { cert = { path = "/etc/pgtm/postgres.crt" }, key = { path = "/etc/pgtm/postgres.key" } }
```

## Verify authorization after enabling TLS

Check a read path:

```bash
pgtm -c config.toml status
```

Then check an admin path:

```bash
pgtm -c config.toml switchover request
```

That verifies both transport reachability and role-token enforcement.

## Troubleshooting

### API starts, but every CLI request is rejected

Check `api.auth` and `pgtm.api.auth` first. Working TLS does not imply valid tokens.

### HTTPS works, but admin operations fail

Confirm you are using an admin token instead of a read-only token.

### Enabling client certificate verification locks out clients

Check the configured `client_ca` bundle and whether callers actually present certificates rooted in that CA.

### PostgreSQL traffic still appears to be plaintext

Check whether `postgres.tls.mode` is still `disabled`, or whether clients still connect with a non-TLS `sslmode`.
