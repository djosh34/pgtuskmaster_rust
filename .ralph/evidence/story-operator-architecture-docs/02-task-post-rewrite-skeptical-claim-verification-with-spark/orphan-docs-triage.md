# Orphan docs triage (fail-closed)

This file is generated during the Task 02 docs verification run.

The `scope-map.csv` build step enumerates all markdown under `docs/src/` and marks anything not reachable from `docs/src/SUMMARY.md` as `orphan` (except `docs/src/SUMMARY.md` itself, which is `internal-only`).

This repository is explicitly **greenfield**. We do not keep legacy doc sets around "just in case" because they create a silent contradiction surface.

## Decisions

All current `orphan` files are **removed** because they are superseded by the new IA under `docs/src/{start-here,quick-start,operator,lifecycle,assurance,interfaces,contributors}`.

Rationale (common):
- these pages are not in the mdBook navigation, so they are easy to forget and drift
- they contain many strong behavioral/safety claims that would require separate verification
- the new IA already covers the same topics in a consolidated, operator-first flow

## Orphan files removed

List source: `orphan-files.txt`.

- `docs/src/architecture/control-loop.md` → remove (superseded by new IA)
- `docs/src/architecture/dcs-keyspace.md` → remove (superseded by new IA)
- `docs/src/architecture/deployment-topology.md` → remove (superseded by new IA)
- `docs/src/architecture/failover-and-recovery.md` → remove (superseded by new IA)
- `docs/src/architecture/ha-lifecycle.md` → remove (superseded by new IA)
- `docs/src/architecture/index.md` → remove (superseded by new IA)
- `docs/src/architecture/node-runtime.md` → remove (superseded by new IA)
- `docs/src/architecture/safety-and-fencing.md` → remove (superseded by new IA)
- `docs/src/architecture/startup-planner.md` → remove (superseded by new IA)
- `docs/src/architecture/switchover.md` → remove (superseded by new IA)
- `docs/src/architecture/system-context.md` → remove (superseded by new IA)
- `docs/src/concepts/index.md` → remove (superseded by Start Here + Assurance)
- `docs/src/concepts/mental-model.md` → remove (redundant with `docs/src/assurance/runtime-topology.md`)
- `docs/src/concepts/roles-and-trust.md` → remove (superseded by Lifecycle + Assurance trust discussion)
- `docs/src/docs-style.md` → remove (superseded by `docs/src/contributors/docs-style.md`)
- `docs/src/operations/config-migration-v2.md` → remove (superseded by Operator Guide configuration)
- `docs/src/operations/deployment.md` → remove (superseded by Operator Guide deployment)
- `docs/src/operations/docs.md` → remove (superseded by Contributors docs style/workflow)
- `docs/src/operations/index.md` → remove (superseded by Operator Guide)
- `docs/src/operations/observability.md` → remove (superseded by Operator Guide observability)
- `docs/src/reading-guide.md` → remove (superseded by `docs/src/start-here/docs-map.md`)
- `docs/src/testing/bdd.md` → remove (superseded by Contributors testing system)
- `docs/src/testing/ha-e2e-stress-mapping.md` → remove (superseded by Contributors testing system)
- `docs/src/testing/harness.md` → remove (superseded by Contributors testing system)
- `docs/src/testing/index.md` → remove (superseded by Contributors testing system)

