# HA Lifecycle

The HA worker models node behavior as an explicit lifecycle of phases.

To keep diagrams readable, it helps to separate:
- steady-state role phases (Replica/CandidateLeader/Primary)
- recovery and safety phases (Rewinding/Bootstrapping/Fencing/FailSafe)

## Steady-state roles
```mermaid
stateDiagram-v2
  [*] --> Replica
  Replica --> CandidateLeader: leader missing\nand promotion is safe
  CandidateLeader --> Primary: leader acquired\nand Postgres ready
  Primary --> Replica: demotion needed\n(switchover or safety)
```

## Recovery and safety phases
```mermaid
stateDiagram-v2
  [*] --> WaitingPostgresReachable
  WaitingPostgresReachable --> WaitingDcsTrusted: Postgres reachable
  WaitingDcsTrusted --> FailSafe: trust degraded\n(but reachable)
  WaitingDcsTrusted --> Replica: trust OK\nand following leader

  Primary --> Fencing: conflicting leader\nor safety trigger
  Fencing --> Replica: demoted safely

  Replica --> Rewinding: timeline divergence
  Rewinding --> Bootstrapping: rewind fails\nor cannot proceed
  Bootstrapping --> Replica: cloned safely
```

These diagrams are deliberately simplified to convey the architecture; the core concept is that **role changes are gated by trust and safety invariants**.

