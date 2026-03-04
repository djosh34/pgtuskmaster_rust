# Unplanned Failover

Failover addresses unplanned primary loss or primary unsafety. Candidate nodes evaluate whether promotion is safe under current trust and readiness evidence.

Promotion is not granted solely because the old leader is unreachable. The node also requires sufficient coordination confidence and local readiness.

## Why this exists

Failover is where optimistic assumptions are most dangerous. The lifecycle intentionally makes promotion conditional to prevent split-brain during partial failures.

## Tradeoffs

Conservative failover can increase recovery time in ambiguous conditions. The benefit is reduced probability of divergent write histories.

## When this matters in operations

During an outage, the key question is not "why is promotion delayed" in isolation. The key question is whether evidence quality supports safe promotion. Use trust, lease, and readiness signals together.
