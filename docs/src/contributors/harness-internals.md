# Harness Internals

The test harness exists to run **real-binary** HA scenarios in a repeatable way.

In this repo, the harness is not “test convenience glue”; it is part of the system’s correctness story. If the harness is flaky, you can’t trust e2e results. If the harness silently skips prerequisites, you get false confidence.

This chapter describes the harness primitives you will interact with when adding or debugging real-binary tests.

## Core harness responsibilities

At a high level, the harness is responsible for:

- creating **isolated namespaces** (directories, log roots, port allocations)
- allocating and **leasing ports** to avoid cross-test interference
- starting and stopping **etcd clusters** as real processes
- starting and stopping **Postgres instances** as real processes
- starting node runtimes and driving lifecycle scenarios
- injecting failures (network blocks/latency) and collecting artifacts for debugging.

The harness lives under `src/test_harness/` and is consumed by real-binary e2e tests (for example HA scenarios in `src/ha/e2e_*.rs`).

## Common pitfalls addressed by harness design

The harness is designed to prevent “classic” e2e failure modes:

- **port races** in parallel test execution
- **stale directories** from prior runs (leading to confusing “already initialized” behavior)
- **incomplete teardown** leaving processes behind (port collisions, slowdowns)
- **path-length and permissions** issues (Unix sockets and PGDATA permissions)
- **unstable startup ordering** masking real HA bugs.

Some of these pitfalls are unavoidable when running real processes; the harness aims to make them deterministic and discoverable.

## Namespaces: every scenario gets its own filesystem root

The namespace utility lives in `src/test_harness/namespace.rs`.

`NamespaceGuard::new(test_name)` creates a unique directory under the system temp dir:

- root: `$TMPDIR/pgtuskmaster-rust/<namespace-id>/`
- subdirs: `logs/` and `run/`.

The namespace id is intentionally unique (includes a timestamp, PID, and a counter) so that parallel tests do not collide.

The guard cleans up the namespace directory on drop. If a test fails and you want to inspect artifacts, the first thing to do is locate the namespace directory path and review the logs inside `logs/`.

## Port allocation and leasing (how we avoid collisions)

Port allocation is in `src/test_harness/ports.rs`.

The harness does two things:

1. **binds actual listeners** on `127.0.0.1:0` to obtain ephemeral ports, and
2. on Unix, it **leases those ports** across processes by writing a small registry file under `/tmp/` protected by a file lock.

The concrete behavior:

- `allocate_ports(n)` returns a `PortReservation` that holds the TCP listeners open, so no other process can bind them.
- on Unix, ports are also leased in `/tmp/pgtuskmaster_rust_port_leases.json` using `flock` to reduce cross-process races between concurrent `cargo test` workers.
- dropping the reservation best-effort releases the leases.

The important sharp edge:

- if you “reserve” a port and then drop the listener before the component binds, another test can steal it.
- the harness therefore prefers passing already-bound listeners into components when possible (see `net_proxy` for the pattern).

## Real binaries are mandatory (and validated)

The harness intentionally fails closed if prerequisites are missing.

Binary validation is enforced by:

- `src/test_harness/binaries.rs`: the public harness entry points (`require_*_for_real_tests(...)`)
- `src/test_harness/provenance.rs`: provenance verification (policy + attestation + hash/version checks)

Real-binary prerequisites are defined in the repo-tracked policy: `tools/real-binaries-policy.json`.

Installers (`./tools/install-etcd.sh` and `./tools/install-postgres16.sh`) generate a local attestation manifest at `.tools/real-binaries-attestation.json` that records:

- expected repo-relative path under `.tools/`
- sha256 and file size
- install timestamp (for debugging).

During tests, the harness verifies that required binaries are:

- regular executable files (symlinks are allowed only if policy explicitly allows them and the canonical target path is in the allowlist),
- contained within `.tools/` after canonicalization (or, for allowlisted symlinks, contained within the allowlisted system prefixes),
- not group/other writable,
- byte-identical to the attested sha256/size, and
- version-compatible with policy (checked via `--version` after hash validation).

If a real-binary test fails with “missing prerequisite”, the fix is to install the binaries (via scripts under `tools/`), not to skip the test.

## Process lifecycle: etcd and Postgres

The harness runs etcd and Postgres as real OS processes and captures logs to namespace directories.

### etcd clusters (`src/test_harness/etcd3.rs`)

The etcd harness can spawn:

- a single etcd instance (`spawn_etcd3`)
- or a multi-member cluster (`spawn_etcd3_cluster`).

It enforces basic sanity before spawning:

- member names must be unique
- client and peer ports must not collide.

Readiness is currently checked by attempting to connect to the client port. If a member exits before readiness or does not become reachable before the timeout, the harness returns a structured error and attempts to shut down any already-started members.

### Postgres 16 instances (`src/test_harness/pg16.rs`)

The Postgres harness spawns `initdb` and `postgres` using the configured binaries.

Key behaviors to know:

- data directories are created under the namespace and rejected if they already exist (stale paths are treated as a test bug)
- on Unix, PGDATA permissions are set to `0700` (Postgres rejects wider permissions)
- `initdb` is currently invoked with `-A trust` and `-U postgres` for test simplicity
- readiness is currently checked by attempting to connect to the TCP port.

Shutdown uses a “TERM then kill” pattern with timeouts so leftover processes do not accumulate.

## Fault injection: TCP proxy links

Network fault injection uses a TCP proxy in `src/test_harness/net_proxy.rs`.

Each `TcpProxyLink`:

- listens on a local address
- forwards to a target address
- supports runtime mode changes:
  - `PassThrough`
  - `Blocked` (drops connections; aborts active tasks)
  - `Latency { upstream_delay_ms, downstream_delay_ms }`.

Proxy listeners run on a dedicated thread with a single-threaded tokio runtime. This is deliberate: it prevents long-running work on the main test runtime from starving the proxy’s accept loop.

## Determinism controls and timeouts

Real-binary e2e tests need explicit time bounds. The harness provides timeouts in many primitives (spawn readiness, shutdown), but scenario code must still:

- use bounded polling loops (with overall deadlines)
- treat “transport errors” as part of the system’s behavior (retry if appropriate, or fail with clear context)
- log enough context to make failures debuggable (namespace id, ports, member names, etc.).

## Adjacent subsystem connections

Harness code exists to exercise real runtime behavior:

- Read [Testing System Deep Dive](./testing-system.md) for how harness-based tests fit into `make test` vs `make test-long`.
- Read [Worker Wiring and State Flow](./worker-wiring.md) to understand which worker channels you should observe or poll in an e2e scenario.
- Read [API and Debug Contracts](./api-debug-contracts.md) for the debug snapshot tooling that makes e2e triage much faster.

## Why this matters

Without harness discipline, HA e2e tests produce false positives and false negatives. Reliable fixtures are part of correctness, not optional testing convenience.
