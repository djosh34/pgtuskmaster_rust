# What Problem This Solves

PostgreSQL high availability fails predictably when built on ad-hoc scripts and implicit assumptions. The primary risk isn't just outage duration; it's unsafe role changes made with incomplete information, leading to split-brain scenarios and divergent histories.

This implementation reduces that risk by channeling all role changes through shared, DCS-trust-aware HA logic. Promotion requires current evidence rather than one-time operator judgment, and planned transitions follow the same control path as unplanned ones.

## What Changes for Operators

Instead of manual leader selection, `pgtuskmaster` provides a continuous control loop:

- Local PostgreSQL state is observed directly
- Shared coordination state is read from etcd
- Operator intent is written into the same coordination model
- Nodes throttle or refuse promotion when evidence is insufficient

In healthy clusters, this yields repeatable transitions. In degraded clusters, it replaces silent optimism with explicit conservatism.

## What That Means in Practice

This project intentionally trades maximum liveness for stronger safety under ambiguity. During network instability, etcd disruption, or partial-cluster failures, the goal isn't "always promote quickly" but "promote only when safety evidence is strong enough."

## Why Ad-Hoc HA Fails Under Pressure

Most unsafe HA incidents begin not with a single bug, but with incomplete evidence treated as certainty. A node loses contact with the coordination service. An operator sees only one side of a network partition. A demotion command executes on stale assumptions. A recovery script assumes the old primary is dead when it's merely unreachable. Each step may seem reasonable in isolation, yet the combined result can yield two writers or an unrecoverable replica.

`pgtuskmaster` absorbs this operational pressure by treating uncertainty as a first-class input. When DCS trust drops, PostgreSQL becomes unreachable, leadership evidence conflicts, or recovery would require rewriting history, the controller slows down rather than assuming missing evidence is probably fine. It maintains a conservative posture until conditions become legible again.

## The Specific Failure Mechanics This Project Tries to Bound

### Ambiguous Leadership

Leadership changes are safe only when the system can distinguish "the old primary is gone" from "the old primary is merely not visible from here." When this distinction blurs, aggressive promotion becomes dangerous. Local health alone doesn't prove write-safety. `pgtuskmaster` therefore uses DCS-backed leadership evidence and trust state as promotion gates rather than relying on a single observer's confidence.

### Stale Coordination Assumptions

The most damaging HA mistakes often stem from stale data that still looks structurally valid: an unrefreshed leader key, a member record from a previous run, or a topology assumption from before a restart. The project makes these coordination records visible through `/ha/state`, debug views, logs, and lifecycle-specific explanations so operators can reason about freshness rather than just presence.

### Unsafe Manual Failover Habits

Manual failover runbooks often compress multiple safety checks into one command: Is the primary truly lost? Is the candidate caught up? Will the demoted node stay down? Can we recover if conditions change? When these questions hide behind a shell script, the script inherits every blind spot of the human invoking it. `pgtuskmaster` separates these checks into explicit phases and decision gates, revealing which precondition is missing rather than treating "no failover yet" as unexplained stubbornness.

### Recovery After Split or Divergence

Even survivable outages create difficult rejoin scenarios. A former primary may require `pg_rewind`, a fresh base backup, or full bootstrap depending on history and visible leader state. The real cost of unsafe promotion isn't just minutes of confusion; it's prolonged repair work plus lingering uncertainty about node trustworthiness. This project prefers bounded conservatism before cut-over rather than cleanup after divergence.

## Operational Consequences of Weak Safety Signals

When safety signals are weak, the cluster may appear slower or more restrictive than optimistic HA tools. A switchover request may be accepted but not complete until a successor appears. A failover may delay until sufficient evidence exists for a safe replacement primary. A node that was writing moments ago may step down or enter fail-safe rather than continue as if coordination were healthy.

These outcomes are deliberate. They prevent the data path from being driven by confidence theater. In practice, operators should interpret hesitation as a signal to inspect trust, leadership records, PostgreSQL reachability, and recovery feasibility rather than immediately forcing another transition. The later Lifecycle, Troubleshooting, and Assurance chapters explain exactly how to perform that inspection.
