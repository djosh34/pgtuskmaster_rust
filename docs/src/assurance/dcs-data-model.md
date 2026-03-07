# DCS Data Model and Write Paths

The DCS namespace is scoped under `/<scope>/...` and stores a small set of coordination records with intentionally narrow ownership. That explicit ownership is part of the safety design. If too many components could write the same key class opportunistically, diagnosing stale or contradictory coordination state would become much harder.

## Core keys and writers

| Key pattern | Typical writer | Purpose |
| --- | --- | --- |
| `/<scope>/member/<member_id>` | DCS worker | publish local member state derived from PostgreSQL observation |
| `/<scope>/leader` | HA worker | leader lease and primary coordination |
| `/<scope>/switchover` | Node API on request, HA worker on clear | planned transition intent and later cleanup |
| `/<scope>/config` | startup/bootstrap path when present | cluster configuration seed record |
| `/<scope>/init` | startup/bootstrap path | initialization lock or cluster-init coordination |

## Why write ownership matters

Each key family answers a different question:

- **member records** answer "what is each node currently publishing about itself"
- **leader record** answers "which member currently claims primary leadership ownership"
- **switchover record** answers "has an operator asked the cluster to evaluate a planned handoff"
- **config/init records** answer startup questions about cluster identity and whether initialization has already happened

The key point is that stale values in those families mean different things. A stale member record often points to DCS worker publication problems or missing PostgreSQL inputs. A stale leader record points to HA lease maintenance or cleanup behavior. A stale switchover record may mean the API write succeeded but the lifecycle never reached the point where it should clear the request, or that the clear path itself failed.

## Update semantics and diagnostic meaning

### Member records

Member records are continuously republished from the DCS worker using current PostgreSQL-derived local state. They are not meant to be hand-edited control inputs. During diagnosis, this means a broken member record usually tells you more about observation or publication health than about HA policy directly.

### Leader record

The leader record is coordinated by the HA path because leadership is an HA decision, not a generic DCS worker fact. This separation matters when failover is delayed. If member records keep updating but leader ownership does not change, the right question is usually about trust, eligibility, or lease behavior rather than whether the DCS worker is alive.

### Switchover record

Switchover is intentionally split across two writers. The API writes intent when an operator requests a switchover. The HA path clears that intent after the lifecycle has handled it. That split reinforces the distinction between accepted intent and completed transition. A present switchover record is therefore not automatically evidence of a stuck API write; it may simply mean the lifecycle has not yet reached the point where clearing it is correct.

### Init and config records

These records mostly matter during startup and bootstrap reasoning. They keep new nodes from deciding independently that they should initialize a brand-new cluster when a cluster for the same scope already exists or is already being initialized elsewhere.

## Cross-subsystem implications

The DCS data model only works because several subsystems interpret it consistently:

- the DCS worker refreshes and publishes the cache view
- the HA worker decides whether current trust and key contents justify leadership actions
- the API may create switchover intent but does not complete the lifecycle on its own
- startup planning inspects initialization and leader evidence before selecting bootstrap, clone, or resume

That means a DCS symptom often spans more than one worker. For example, a stale switchover record may be created by a valid API write, left in place because the HA lifecycle is still waiting for safe successor conditions, and remain visible through the API even while the operator wonders whether the request "stuck." The record is not wrong merely because it still exists; its meaning depends on the rest of the lifecycle state.

## How to use this during diagnosis

Explicit ownership makes diagnosis more disciplined:

- stale member record: start with the DCS worker and its PostgreSQL inputs
- stale leader record: inspect HA trust, lease logic, and current phase
- stale switchover record: distinguish request acceptance from lifecycle completion
- surprising init/config record behavior: inspect startup assumptions, scope, and bootstrap history

That is the core assurance value of the DCS model. It turns "the store looks odd" into a smaller, more specific question about which part of the system owns the oddness and what kind of safety implication it actually has.
