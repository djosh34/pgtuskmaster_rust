# Unplanned Failover

Failover is driven by observed state. When coordination trust is sufficient and PostgreSQL is reachable, candidate or replica nodes can acquire leadership and promote. When trust is degraded they move to conservative phases such as fail-safe instead of promoting.

Promotion is not granted simply because the old leader is unreachable. It also requires sufficient coordination trust, local SQL reachability, and lease or leadership evidence.

## Promotion requirements

During an outage, the question is not only "why is promotion delayed." The real question is whether the node has enough evidence to promote safely. In this implementation that means:

- acceptable coordination trust
- local PostgreSQL reachability
- lease and leadership predicates that do not imply a conflicting primary

## What delayed failover usually means

Conservative failover can increase recovery time under ambiguous conditions. That is the intended tradeoff. When failover is delayed, start by checking:

- `dcs_trust`
- current leader visibility in DCS
- local PostgreSQL readiness on the candidate node
- recent HA and process events that explain which guard is still failing
