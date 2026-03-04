# Harness Internals

The harness provides deterministic primitives for multi-node HA scenarios using real binaries.

## Core harness responsibilities

- create isolated namespaces (ports, directories, sockets)
- start and stop etcd clusters
- start and control node runtimes
- provide helper APIs for role and trust polling
- collect artifacts for debugging failures

## Common pitfalls addressed by harness design

- port races during parallel tests
- path-length issues for Unix sockets
- incomplete teardown leaving stale processes
- unstable ordering in multi-node startup

## Why this matters

Without harness discipline, HA e2e tests produce false positives and false negatives. Reliable fixtures are part of correctness, not optional testing convenience.
