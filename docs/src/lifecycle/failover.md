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

## Missing leader evidence is not enough by itself

The most common failover misunderstanding is to treat leader absence as a sufficient promotion signal. In this project it is only one input. A node can be unable to see a leader for many reasons: real leader loss, stale DCS data, its own degraded coordination view, or a partial partition. Promotion therefore stays coupled to trust and local readiness instead of treating every missing-leader observation as "safe to take over".

This shows up directly in the phase logic:

- a replica without a believable active leader can move toward candidate-leader behavior
- a candidate leader still has to win leadership rather than assuming it already has it
- if trust is not full quorum, the node does not continue along the normal promotion path

## Decision gates during the transition

### Trust gate

If the DCS is not trusted, the node does not treat its own outage interpretation as authoritative enough for ordinary failover. This is the most important reason a failover can be delayed while still being correct.

### Local PostgreSQL gate

A node that cannot reach its own PostgreSQL process is not a safe promotion target. Even if it has a promising coordination picture, it still needs a usable local database before becoming primary. That is why waiting-for-Postgres and recovery phases can appear in the middle of what operators hoped would be a fast failover.

### Leadership gate

Leadership is not merely an API label. The node must either observe that it already owns the leader record or keep attempting leadership until it wins without conflicting active evidence. If another active leader is visible, following or recovery remains safer than self-promotion.

## Why delayed failover can still be the right outcome

There are real incidents where immediate promotion would restore writes faster. There are also real incidents where immediate promotion would create two writers or would promote a node that cannot recover cleanly once the network picture clears. `pgtuskmaster` chooses the second risk as the one to avoid.

That means delayed failover often indicates the system is refusing to translate uncertainty into confidence. Operators should read that delay as a request for better evidence: clearer trust, healthier local PostgreSQL, or a more coherent leader picture.

## Operator-visible consequences

When failover is progressing normally, you should see a coherent story across surfaces:

- the old leader stops looking like an active primary
- a candidate or replica becomes a more plausible promotion target
- `/ha/state` reflects candidate or primary-oriented decisions that fit the observed topology
- logs and debug history show convergence rather than random churn

When failover is not progressing, look for which proof is missing rather than which node you want to promote. The runtime is answering "what is safe now", not "what would be convenient if my current guess were correct".
