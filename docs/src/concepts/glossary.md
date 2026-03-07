# Glossary

## DCS

Distributed configuration store used for shared coordination. In the current implementation that store is etcd. The DCS is where leadership, member visibility, and operator intent records live. It contributes evidence to the HA decision loop, but it is not the only source of truth; local PostgreSQL observation still matters.

## Scope

Namespace prefix used for cluster-owned DCS keys, usually shaped like `/<scope>/...`. Scope keeps one cluster's coordination records separate from another's. When a node reports state through the API, the scope tells you which coordination namespace it believes it is operating in.

## Member

One node identity participating in cluster coordination. A member is more than a container or host name. It is the identifier used in DCS records, HA state payloads, and leadership logic. When member identity is wrong or inconsistent, many other surfaces become hard to interpret.

## Leader record

The DCS record that names which member currently owns primary leadership. Operators should read it as coordination evidence, not as a magical guarantee that the named node is definitely writable right now. The HA loop still has to reconcile the leader record with local PostgreSQL reachability and trust level.

## Switchover intent

The operator request record for a planned primary transition. In this project, switchover is not a hidden side channel. It is written into the same coordination model the HA loop already uses, which means accepted intent still has to pass the same safety checks as any other role change.

## Trust

The current confidence level in coordination data quality. Trust determines how willing the controller should be to act on DCS-backed evidence. When trust degrades, the runtime becomes more conservative because it no longer treats the coordination picture as strong enough for ordinary role transitions.

## Fail-safe

A conservative operating posture entered when coordination trust is degraded. Fail-safe is not a synonym for "everything is broken". It means the node is intentionally limiting risky HA behavior because the evidence needed for confident action is no longer strong enough. This often protects safety at the cost of liveness.

## Fencing

Safety behavior used to reduce concurrent-writer risk when conflicting leadership evidence appears. Fencing is an intentionally restrictive phase. Operators should interpret it as a protective response to ambiguity, not as routine steady-state behavior.

## Bootstrap

Initial data and role setup path at startup. Depending on what the node observes, bootstrap may mean creating a fresh local PostgreSQL instance, joining an existing leader through a base backup path, or resuming from coherent local state. The Lifecycle chapter explains the decision branches in detail.

## Rewind

Divergence-recovery path using `pg_rewind`. Rewind is relevant when a node has usable local state but that state no longer matches the current leader's history closely enough for safe rejoin as-is. It is usually less destructive than a full re-bootstrap, but it still depends on the right evidence and prerequisites being present.

## Bootstrap recovery

Local reinitialization path used when rewind is unsafe, impossible, or already failed. In the implementation and docs this refers to the node rebuilding local PostgreSQL state rather than trying to preserve divergent data files. Base backup cloning is related but distinct: it is the path for joining an existing leader with a fresh copy rather than reusing local history.
