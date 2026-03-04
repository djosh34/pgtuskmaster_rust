# HA Lifecycle

The HA worker models node behavior as an explicit lifecycle of phases.

To keep diagrams readable, it helps to separate:
- steady-state role phases (Replica/CandidateLeader/Primary)
- recovery and safety phases (Rewinding/Bootstrapping/Fencing/FailSafe)

## Steady-state roles
```mermaid
stateDiagram-v2
  [*] --> Replica
  Replica --> CandidateLeader: leader unavailable<br/>and DCS trust is OK
  CandidateLeader --> Primary: leader acquired<br/>and Postgres reachable
  Primary --> Replica: demotion needed<br/>(switchover or safety)
```

## Recovery and safety phases
```mermaid
stateDiagram-v2
  [*] --> WaitingPostgresReachable
  WaitingPostgresReachable --> WaitingDcsTrusted: Postgres reachable
  WaitingDcsTrusted --> FailSafe: DCS trust degraded
  WaitingDcsTrusted --> Replica: trust OK<br/>and following leader

  Primary --> Fencing: conflicting leader<br/>or safety trigger
  Fencing --> Replica: demoted safely

  Primary --> Rewinding: local SQL unhealthy<br/>(primary safety trigger)
  Rewinding --> Bootstrapping: rewind fails<br/>(or is unsafe)
  Bootstrapping --> Replica: re-bootstrapped safely
```

These diagrams are deliberately simplified to convey the architecture; the core concept is that **role changes are gated by trust and safety invariants**.
