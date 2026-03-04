# Tradeoffs and Limits

Every HA design embodies tradeoffs. This system is explicit about prioritizing safety under uncertainty.

## Primary tradeoffs

- Safety over immediate availability in ambiguous coordination states.
- Explicit configuration over permissive defaults.
- Reconciliation loop discipline over ad-hoc one-shot transitions.

## Practical limits

- Coordination quality depends on etcd health and consistent scope usage.
- Recovery speed depends on rewind/bootstrap prerequisites and network access.
- Incorrect auth or path config can block otherwise valid lifecycle transitions.

## Operational interpretation

A conservative decision is not automatically a bug. In many cases it is the intended safe behavior under current evidence quality.

## Next step

Use this chapter with [System Lifecycle](../lifecycle/index.md) and [Troubleshooting by Symptom](../operator/troubleshooting.md) to differentiate true defects from protective behavior.
