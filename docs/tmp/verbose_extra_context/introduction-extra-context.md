# Extra context for docs/src/explanation/introduction.md

There is no `README.md` or `PROJECT_OVERVIEW.md` in the repository root. Any introduction draft must therefore derive the project overview from the codebase and the already-published docs rather than claiming a separate high-level project statement exists.

What the project is, based on source and published docs:

- The crate name is `pgtuskmaster_rust`.
- `src/lib.rs` exposes modules for API, CLI, configuration, distributed coordination (DCS), PostgreSQL state inspection, runtime orchestration, and worker state handling.
- The published docs already describe the product as a PostgreSQL high-availability controller that manages cluster state through an HTTP API and a distributed coordination service backed by etcd.
- Existing tutorials and how-to pages assume an operator runs one pgtuskmaster node alongside each PostgreSQL node and uses etcd as the shared coordination system.
- The architecture and failure-mode docs show that the system is intentionally opinionated about safety: DCS trust is part of leadership gating, not just service discovery.

Primary capabilities visible in code and docs:

- Runtime config defines cluster identity, DCS endpoints and scope, PostgreSQL process/binary paths, HA timing, logging, API security, and debug settings.
- The API exposes HA state, switchover operations, and debug endpoints.
- The CLI client `pgtuskmasterctl` talks to the HTTP API rather than reaching into DCS directly.
- The runtime starts PostgreSQL, monitors local state, publishes membership into DCS, and runs HA decision logic that can become primary, follow a leader, recover a replica, fence a node, or enter fail-safe.
- The debug subsystem can emit snapshots and timelines for troubleshooting.
- The test harness contains substantial real-binary HA scenarios, indicating the project is designed around end-to-end operational behavior rather than isolated logic only.

Design goals and trade-offs that are supportable from repository evidence:

- Safety over convenience: the DCS trust model can force `FailSafe` or `NotTrusted` states when etcd is unhealthy or member freshness is insufficient.
- Explicit operator observability: `/ha/state`, `/debug/snapshot`, `/debug/verbose`, and `/debug/ui` exist specifically to expose cluster behavior.
- A single node-local runtime config model is used to describe one node's behavior in the cluster.
- Operational workflows are expected to be driven through HTTP and CLI surfaces, not by manual DCS mutation after startup.
- Docker examples focus on etcd plus one or more PostgreSQL/pgtuskmaster nodes, so the minimum documented deployment shapes are single-node and multi-node HA clusters.

Comparisons to alternatives:

- The repository does not contain authoritative comparison text against Patroni, repmgr, or other tools.
- A draft may mention only cautious, source-supported comparisons such as "this project uses etcd-backed coordination, an HTTP API, and explicit trust-gated HA decisions", but it must not invent a marketing comparison matrix or claim goals not demonstrated by the code.

Target audience supportable from repository evidence:

- Operators deploying PostgreSQL HA clusters.
- Contributors and test authors working on HA behavior.
- The docs use operator-oriented phrasing and Docker-based examples, so the audience is broader than library users and is not primarily an embedded Rust API audience.
