Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Draft a new explanation page.

[Output path]
- docs/src/explanation/tls-posture.md

[Page title]
- # Why TLS remains operator-supplied but runtime-enforced

[Audience]
- Readers trying to understand the project's security boundary around certificates, modes, and managed runtime files.

[User need]
- Understand why TLS configuration is validated and enforced by runtime code without turning the project into a certificate-management system.

[mdBook context]
- Link naturally to TLS reference, runtime config, HTTP API reference, and managed PostgreSQL runtime files.

[Diataxis guidance]
- Explanation only.

[Verified facts that are true]
- build_rustls_server_config returns Ok(None) when TLS mode is Disabled.
- When TLS mode is Optional or Required, tls.identity must be configured or configuration fails.
- Client auth can be configured from a client CA and may allow unauthenticated clients when require_client_cert is false.
- The API worker can be configured with Disabled, Optional, or Required TLS modes and requires a server config for Optional or Required.
- For PostgreSQL managed runtime files, production TLS credentials are operator-supplied and pgtuskmaster only copies them into managed runtime files under PGDATA before PostgreSQL starts.
- The managed PostgreSQL config header explicitly states that production TLS material must be supplied by the operator.

[Relevant repo grounding]
- src/tls.rs
- src/api/worker.rs
- src/postgres_managed.rs
- src/postgres_managed_conf.rs

[Design tensions to explain]
- Why the system validates and wires TLS aggressively but avoids inventing security-sensitive material.
- Why API TLS and PostgreSQL TLS share a posture but not the same runtime wiring.
- Why Optional mode still requires identity material.

[Required structure]
- Explain the operator-supplied boundary.
- Explain runtime enforcement versus secret generation.
- Explain the practical consequences for API access and managed PostgreSQL files.

[Facts that must not be invented or changed]
- Do not claim certificate rotation automation exists.
- Do not claim TLS is always required everywhere.
