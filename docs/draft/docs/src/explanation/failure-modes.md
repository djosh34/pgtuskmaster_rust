# Failure Modes and Recovery Behavior

PostgreSQL high availability clusters must survive component failures without violating safety guarantees. Understanding failure modes helps operators choose recovery actions and validate system behavior under stress.

## Detection Pathways

The system uses two distinct detection pathways that operate at different observability levels:

[diagram about detection pathways showing control plane vs host-side observation, **control plane observes via DCS state while host-side observes via member-local pgtm status**]
// todo: Replace the placeholder with either valid mermaid or plain markdown. Do not leave bracketed artifact text in the final doc.

**Control Plane Detection**  
The control plane observes member state through the distributed consensus store. Missing source support for specific implementation details about how process death, network partitions, or disk exhaustion are detected via DCS.
// todo: This throws away source-backed content already present in the existing page. Restore the supported DCS trust model details and remove unsupported examples like disk exhaustion.

**Host-Side Detection**  
Host-side tools query members directly without traversing the cluster network. The `pgtm status --json` command returns per-member state including PostgreSQL role, replication lag, and identity information. This pathway functions even when a member cannot reach the DCS.
// todo: Tighten this to supported facts from the harness code and existing doc. Avoid unsupported wording like "replication lag" if you cannot back it from the provided source set.

The HA acceptance harness relies entirely on host-side detection to enforce safety invariants, making it a useful model for understanding direct observation.

## Safety Invariants

The most critical safety property is the primary count invariant: at most one member may claim the PostgreSQL primary role at any moment. Violations indicate split-brain and risk data corruption.

### Perpetual Invariant Runner

The HA test harness implements a **perpetual primary-count invariant runner** that continuously validates this property across all scenarios. This runner:

- Polls every member individually via `pgtm status --json` through a host-side observer
- Examines only the self-reported `NodeState.pg` field (Primary, Replica, or Unknown)
- Treats command failure as absence of self-report for that member
- Fails the scenario immediately if primary count exceeds one
- Writes a structured artifact at `artifacts/primary-count-invariant-violation.json` containing the violating sample

[diagram about invariant runner lifecycle showing startup before bootstrap, continuous polling, immediate failure on violation, and artifact capture, **runner starts during harness initialization, runs in background thread, writes violation sample to json artifact**]
// todo: Replace the placeholder with valid mermaid or plain markdown. If a diagram is kept, ensure it is source-backed and mdbook-lint-safe.

The runner starts before cluster bootstrap and stops during scenario cleanup, ensuring it validates the entire scenario lifetime. The artifact includes:

```json
{
  "observed_at_ms": 123456789,
  "allowed_primary_counts": [0, 1],
  "primary_count": 2,
  "members": [
    {"member": "node-a", "self_report": {"kind": "primary"}},
    {"member": "node-b", "self_report": {"kind": "primary"}}
  ]
}
```

This design removes the need for feature-local dual-primary assertions or timeline-based bookkeeping, centralizing safety validation in a single always-on component.

### Production vs. Test Behavior

The perpetual runner validates test scenarios but is **not part of production runtime logic**. Missing source support details about whether production uses an analogous continuous invariant checker or relies on control-plane reconciliation loops.
// todo: Rephrase this using the already-supported distinction in the existing page: the runner is a host-side HA test harness safety check, not production runtime behavior. Do not add speculation about missing production analogs.

## Recovery Behavior

**Automatic Restart**  
Missing source support for process restart behavior after unexpected PostgreSQL postmaster exit.
// todo: The previous page had source-backed recovery detail. Restore the supported content instead of replacing the section with generic "missing source support" placeholders.

**Automatic Failover**  
Missing source support for conditions that trigger automatic failover and how the HA decision engine selects a new primary.
// todo: Same issue here. Use the existing source-backed recovery explanation and fit the new invariant-runner explanation into it, rather than deleting recovery detail.

**Operator Tooling**  
The `pgtm` CLI provides `status`, `switchover request`, and other commands for manual intervention. Role-based authentication and TLS verify-full mode protect these operations.

**Switchover Requirements**  
Missing source support for pre-switchover health checks, quiescing writes, or ensuring replica readiness.

## Test Harness Validation

The acceptance suite exercises failure modes in containerized topologies (three-node clusters with shared or per-node DCS). Key features:

- **Fault Injection**: Network partitions, DCS connectivity loss, disk exhaustion, and process kills
- **Continuous Observation**: The invariant runner plus timeline snapshots capture state evolution
- **Artifact Capture**: Logs, compose state, and per-member status are persisted on failure
// todo: Remove unsupported "disk exhaustion" unless you can back it from the provided task/code context. The current HA harness source set here clearly supports network faults, DCS faults, process kills, restart/rejoin flows, and blocker-based recovery faults.

The perpetual invariant runner ensures every scenario enforces the primary-count safety property without burdening feature text with repeated assertions.

[diagram about test harness layers showing given fixture, materialized compose, observer queries, invariant runner, and artifact capture, **each layer transforms test inputs into observable state while the invariant runner validates safety**]
// todo: Replace the placeholder with valid mermaid or plain markdown, or drop the diagram if it does not improve factual clarity.
