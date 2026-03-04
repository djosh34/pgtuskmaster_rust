# Safety and Fencing

Safety is the system’s “brake”: when signals are inconsistent, the node should prefer actions that reduce split-brain risk.

Two different situations must be kept distinct:
1. “Leader information is missing/unavailable” → affects follow/promotion decisions
2. “Conflicting leader information exists” → indicates split-brain risk and should trigger fencing

```mermaid
flowchart TD
  A[Observe DCS leader record] --> Missing{Leader missing?}
  Missing -->|yes| PromoteCheck["Consider promotion\n(only if safe)"]
  Missing -->|no| Conflict{"Leader record conflicts\nwith local invariants?"}
  Conflict -->|yes| Fence[Fencing / demotion path]
  Conflict -->|no| Follow[Follow leader / stay stable]
```

Fencing is not a punishment; it is a safety mechanism.
It is acceptable for the system to become less available temporarily if that prevents two primaries from accepting writes concurrently.
