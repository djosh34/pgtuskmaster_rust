# Recovery and Rejoin

After failover or fencing events, nodes may need recovery work before they can safely follow or become eligible again.

Common recovery paths:
- rewind when divergence is recoverable
- bootstrap when rewind is unsafe or unavailable
- rejoin as replica after data and coordination state are coherent

## Why this exists

A node that was previously primary can carry divergent history. Recovery and rejoin behavior is driven by explicit process outcomes (success/failure/timeout) plus DCS trust and leader reachability, and it requires a recovery action before rejoin when divergence risk exists.

## Tradeoffs

Recovery work increases transition time and may require stronger prerequisites (auth, connectivity, binary tool availability). The benefit is cleaner data lineage and safer future elections.

## When this matters in operations

If a node repeatedly fails to rejoin, treat identity, replication auth, and rewind connectivity as first-class diagnostics. The runtime surfaces these issues through process job outcomes and errors; avoid forcing eligibility until recovery preconditions are satisfied.
