# Startup Planner

Before the node enters steady-state reconciliation, it runs a startup planner once to decide what kind of initialization path is safe.

At a high level, there are three startup outcomes:
- `InitializePrimary`: create/initialize a new primary instance
- `CloneReplica`: clone from an existing primary to become a replica
- `ResumeExisting`: reuse an existing local data directory and recover safely

```mermaid
flowchart TD
  Start[Node starts] --> HasData{Local data dir exists?}
  HasData -->|yes| Resume[ResumeExisting]
  HasData -->|no| HasLeader{DCS has evidence of a healthy leader?}
  HasLeader -->|yes| Clone[CloneReplica]
  HasLeader -->|no| Init[InitializePrimary]
```

Why this separation matters:
- Startup decisions affect what “role” the node can safely enter later.
- Inconsistent coordination (trust degraded) should bias toward safer, more conservative startup paths.
