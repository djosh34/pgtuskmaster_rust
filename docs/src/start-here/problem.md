# What Problem This Solves

PostgreSQL high availability fails in predictable ways when operations rely on ad-hoc scripts and implicit assumptions. The common failure mode is not only outage duration. The larger risk is unsafe role changes under partial information, which can lead to concurrent primaries and divergent histories.

This implementation reduces that risk by funneling role changes through shared, DCS-trust-aware HA logic. Promotion is gated by current evidence instead of a one-time operator guess, and planned transitions flow through the same control path as unplanned ones.

## What changes for operators

Instead of treating leader selection as a manual procedure, `pgtuskmaster` turns it into a continuous control loop:

- local PostgreSQL state is observed directly
- shared coordination state is read from etcd
- operator intent is written into the same coordination model
- the node slows down or refuses promotion when the evidence is weak

In a healthy cluster that gives you repeatable transitions. In a degraded cluster it gives you explicit conservatism instead of silent optimism.

## What that means in practice

The project intentionally trades maximum liveness for stronger safety under ambiguity. During network instability, etcd disruption, or partial-cluster failures, the correct expectation is not "always promote quickly." The correct expectation is "promote only when safety evidence is strong enough."

## Why ad-hoc HA fails under pressure

Most unsafe HA incidents do not begin with a dramatic single bug. They begin with incomplete evidence being treated as certainty. A node loses contact with a coordination service. An operator only sees one side of a partition. A demotion command is issued based on stale assumptions. A recovery script assumes the old primary is definitely dead when it is only temporarily unreachable. Each local step may look reasonable in isolation, but the combined outcome can still produce two writers or a replica that can no longer rejoin cleanly.

That is the operational pressure `pgtuskmaster` is designed to absorb. Instead of encoding the happy path only, it treats uncertainty itself as a first-class input. If DCS trust drops, if PostgreSQL is unreachable, if leadership evidence conflicts, or if recovery would require history to be rewritten, the controller does not continue as though the missing evidence is probably fine. It shifts into a slower and more conservative posture until the conditions become legible again.

## The specific failure mechanics this project tries to bound

### Ambiguous leadership

Leadership changes are safe only when the system can distinguish "the old primary is gone" from "the old primary is merely not visible from here". When that distinction is fuzzy, aggressive promotion logic becomes dangerous. The local node may be healthy, but health alone is not enough to prove it is safe to accept writes. `pgtuskmaster` therefore uses DCS-backed leadership evidence and trust state as promotion gates instead of relying on a single observer's confidence.

### Stale coordination assumptions

The most damaging HA mistakes often come from stale data that still looks structurally valid. A leader key that has not been refreshed, a member record from an earlier run, or a remembered topology assumption from before a restart can all point operators in the wrong direction. The project makes those coordination records visible through `/ha/state`, debug views, logs, and lifecycle-specific explanations so that operators can reason about freshness rather than merely presence.

### Unsafe manual failover habits

Manual failover runbooks often compress several separate questions into one command: is the current primary really lost, is a candidate actually caught up enough, will the demoted node stay down, and can the result be recovered later if conditions change? When those questions are hidden behind a shell script, the script inherits every blind spot of the human invoking it. `pgtuskmaster` separates those checks into explicit phases and decision gates so operators can see which precondition is missing instead of treating "no failover yet" as unexplained stubbornness.

### Recovery after split or divergence

Even when an outage is survivable, rejoining the cluster after a role mistake can be the hardest part. A former primary may require `pg_rewind`, a fresh base backup, or a full bootstrap depending on the history and the visible leader. The real cost of unsafe promotion is therefore not just a few minutes of confusion; it can be prolonged repair work plus uncertainty about whether recovered nodes are trustworthy. That is why this project prefers bounded conservatism before the cut-over instead of cleanup after divergence.

## Operational consequences of weak safety signals

When safety signals are weak, the cluster may look slower or more restrictive than operators expect from optimistic HA tooling. A switchover request may be accepted but not complete until a successor is visible. A failover may be delayed because there is not yet enough evidence to name a safe replacement primary. A node that was writing moments ago may step down or enter fail-safe rather than continue as if coordination were healthy.

Those outcomes are intentional. They protect the data path from being driven by confidence theater. In practice, that means operators should interpret hesitation as a signal to inspect trust, leadership records, local PostgreSQL reachability, and recovery feasibility instead of immediately forcing another transition. The later Lifecycle, Troubleshooting, and Assurance chapters explain exactly how to perform that inspection.
