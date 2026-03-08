# Why the control API and debug API are separate

pgtuskmaster exposes two HTTP-adjacent surfaces because it has two different jobs to serve: carrying control intent and exposing system understanding.

The [HTTP API reference](../reference/http-api.md) documents the narrow control surface. The [debug API reference](../reference/debug-api.md) documents the richer snapshot surface. This page explains why they are not the same thing.

## Control intent should stay narrow

The control API exists to carry supported operator intent such as observing HA state and creating or clearing switchover requests. That narrowness matches the post-start policy: tests are allowed to observe through `GET /ha/state` and send supported switchover requests, but not to steer internals directly.

Keeping the control API small makes policy easier to enforce. It reduces the temptation to add low-level escape hatches that bypass the runtime's normal coordination path.

## Debugging needs a wider view

The debug API solves a different problem. It assembles `SystemSnapshot` values from config, pginfo, DCS, process, and HA state, records a monotonically increasing sequence, and keeps bounded change and timeline history.

That surface is about explanation and diagnosis. It helps operators and engineers answer "what changed?" and "which domain changed first?" without turning those introspection details into part of the supported control surface.

## Why one surface should not absorb the other

If the public control API absorbed the full debug surface, observability detail would start to look like supported control contract. If the debug surface absorbed control, a tool meant for visibility would become another route for steering internals.

By separating them, the project keeps two promises distinct:

- the control API is the deliberate path for supported actions
- the debug API is the deliberate path for rich introspection

## The tradeoff

Two surfaces mean two places to understand, secure, and operate. The payoff is cleaner policy. Control remains intentionally narrow, while introspection can stay rich without quietly becoming a second control plane.
