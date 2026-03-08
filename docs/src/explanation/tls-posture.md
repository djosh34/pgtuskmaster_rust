# Why TLS remains operator-supplied but runtime-enforced

pgtuskmaster treats TLS as something the runtime must validate and wire correctly, but not something the runtime is allowed to invent.

The [TLS reference](../reference/tls.md) describes the configuration and runtime hooks. This page explains the security boundary behind them.

## The operator owns credential material

When API TLS mode is `Optional` or `Required`, identity material must be supplied or configuration fails. For managed PostgreSQL runtime files, production TLS material is also operator-supplied and is only copied into managed runtime paths before PostgreSQL starts.

That boundary keeps certificate issuance, CA policy, and secret lifecycle outside the runtime's remit. The project refuses to blur "securely use credentials" into "manufacture credentials for the operator".

## The runtime still enforces posture

Refusing to invent credentials does not mean TLS is soft-optional at runtime. The runtime validates that the chosen mode and the supplied material agree with each other. It can also enforce client-auth behavior from configured CA material and `require_client_cert` settings.

This is why optional TLS still requires identity material. "Optional" describes client negotiation posture, not permission to run without a valid server identity once TLS support has been enabled.

## Why API TLS and PostgreSQL TLS feel similar but are wired differently

The API worker loads TLS into Rustls server configuration. Managed PostgreSQL receives copied files through the runtime-managed file set. The security posture is shared, but the operational wiring follows the responsibilities of the two systems being protected.

In both cases, pgtuskmaster is the enforcer and transporter of supplied material, not the source of truth for creating that material.

## The tradeoff

This design makes the runtime stricter about configuration while deliberately smaller in secret-management scope. Operators keep control over issuance and rotation. The runtime keeps control over whether the supplied posture is complete enough to run safely.
