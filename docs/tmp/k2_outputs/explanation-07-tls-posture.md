# Why TLS remains operator-supplied but runtime-enforced

## The operator-supplied boundary

The project treats TLS credentials as an operator responsibility. Production TLS material must be supplied by the operator; the system never generates certificates, keys, or CAs. This boundary exists because managing security-sensitive material requires infrastructure and policy decisions outside the project's scope.

For the API worker, `build_rustls_server_config` returns `Ok(None)` only when TLS mode is `Disabled`. In `Optional` or `Required` modes, configuration fails immediately if `tls.identity` is missing. The same principle applies to PostgreSQL: the managed runtime file header explicitly states that production TLS credentials are operator-supplied, and pgtuskmaster only copies pre-existing material into `PGDATA` before PostgreSQL starts.

Client authentication follows a similar pattern. The operator supplies a client CA, and the system respects the `require_client_cert` flag: when false, unauthenticated clients are allowed even in TLS-enabled modes.

## Runtime enforcement versus secret generation

Validation is aggressive, but material creation is deliberately absent. The runtime enforces policy—presence of identity, consistency of modes, correct file placement—without venturing into secret lifecycle management. This separation keeps the project's security surface minimal while still catching configuration errors early.

Optional mode requires identity material for this reason: enforcement does not relax when TLS is negotiable. The presence of a certificate must be validated regardless of whether every client will use it.

## Practical consequences for API access and managed PostgreSQL files

API access and PostgreSQL share the same posture but not the same wiring. Both rely on operator-supplied credentials and fail fast if those credentials are missing. However, the API worker loads its configuration directly into a Rustls server config, while PostgreSQL receives copies of the same material through managed runtime files. The system orchestrates the placement and validation but never holds authority over issuance or renewal.

This design means operators retain full control over rotation, CA hierarchies, and provisioning workflows. The project's role is limited to ensuring that whatever material is supplied is used correctly and consistently.
