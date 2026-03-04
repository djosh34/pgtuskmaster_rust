# Unplanned Failover

Failover is driven by observed state. When coordination trust is sufficient and PostgreSQL is reachable, candidate/replica nodes can acquire leadership and promote; when trust is degraded they move to conservative phases (like fail-safe) instead of promoting.

Promotion is not granted simply because the old leader is unreachable. It also requires sufficient coordination trust, local SQL reachability, and lease/leadership evidence.

## Why this exists

Failover is where optimistic assumptions are most dangerous. The lifecycle intentionally makes promotion conditional to prevent split-brain during partial failures.

## Tradeoffs

Conservative failover can increase recovery time in ambiguous conditions. The benefit is reduced probability of divergent write histories.

## When this matters in operations

During an outage, the key question is not "why is promotion delayed" in isolation. The key question is whether evidence quality supports safe promotion. In this implementation that is expressed through explicit trust, local SQL reachability, and lease/leadership predicates.
