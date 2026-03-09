# Introduction

PGTuskMaster is a PostgreSQL high-availability controller. A node runs alongside a PostgreSQL instance, watches local database state, publishes cluster state through a distributed coordination service, and exposes an HTTP API for observation and operator actions.

The repository structure shows that the project is built around a small set of cooperating concerns:

- `config`: runtime configuration for one node
- `dcs`: distributed coordination state and trust evaluation
- `pginfo`: local PostgreSQL state inspection
- `ha`: leader/follower decision logic
- `process`: PostgreSQL lifecycle operations such as bootstrap, rewind, and fencing
- `api` and `cli`: operator-facing control and observation surfaces
- `debug_api`: deeper snapshot and timeline inspection

## What Problem It Solves

Running PostgreSQL in high availability mode is not just a question of promoting a replica. A control plane has to answer harder questions continuously:

- Is the distributed coordination service healthy enough to trust?
- Is the current leader still fresh and reachable?
- Is the local PostgreSQL instance primary, replica, or unknown?
- Should this node promote, follow, rewind, bootstrap, or fence itself?
- How can an operator observe those decisions without guessing?

PGTuskMaster addresses that by combining local PostgreSQL inspection, DCS-backed cluster state, and an explicit HA decision engine. The result is an operator-visible runtime that can explain what it is doing and why.

## Safety Model

The existing architecture and failure-mode documentation make one design choice very clear: DCS trust is part of the safety boundary, not an optional convenience.

The DCS worker classifies trust into three states:

- `full_quorum`: etcd is healthy and enough fresh member information is available
- `fail_safe`: etcd is reachable, but the node does not have enough fresh information to behave normally
- `not_trusted`: etcd itself is unhealthy or unreachable

That trust state feeds directly into HA decisions. A node does not simply "try promotion and see what happens". It first decides whether the cluster view is trustworthy enough to allow leadership behavior at all.

## Runtime Shape

The runtime configuration describes one node in the cluster. It includes:

- cluster identity and member ID
- PostgreSQL paths, ports, and role credentials
- DCS endpoints and cluster scope
- HA timing
- process timeouts and binary locations
- logging sinks
- API security
- debug endpoint enablement

The Docker examples under `docker/configs/` show how the same schema is used for single-node and multi-node deployments by changing per-node identity and topology values.

## Operator Surfaces

Operators interact with the system in two main ways.

The HTTP API exposes:

- `/ha/state` for cluster and decision observation
- `/switchover` and `/ha/switchover` for planned leadership changes
- `/debug/snapshot`, `/debug/verbose`, and `/debug/ui` for deeper diagnostics

The CLI client `pgtm` is a thin client for that API. It is for issuing requests and reading state, not for bypassing the runtime with direct DCS manipulation.

## Deployment Expectations

The published tutorials and compose files show the currently documented deployment shapes:

- a single-node stack with PostgreSQL, pgtuskmaster, and etcd
- a multi-node HA cluster with one pgtuskmaster node per PostgreSQL node and a shared etcd service

The repository also contains a substantial real-binary HA test harness. That is important context: this project is not documented or tested as a lightweight mock-only orchestration layer. It is built and exercised as a runtime that manages real PostgreSQL and etcd processes.

## How To Read The Rest Of The Docs

Use the rest of the book by intent:

- Start with the tutorials when you want to get a cluster running and watch it behave.
- Use the how-to guides when you have an operational goal such as bootstrapping, checking health, switching over, or debugging a live incident.
- Read the explanation pages when you want to understand why the system behaves the way it does.
- Keep the reference pages open when you need exact API fields, CLI flags, or configuration keys.
