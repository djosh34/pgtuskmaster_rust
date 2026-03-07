# Initial Validation

Treat the first launch as incomplete until you can explain what the node is doing from outside the process.

Run through this checklist:

- `pgtuskmasterctl ha state` returns a coherent response instead of timing out or failing auth unexpectedly.
- `dcs_trust`, `ha_phase`, and `ha_decision` make sense for the environment you just started.
- The local PostgreSQL instance is reachable on the configured socket or listen address.
- The etcd scope contains the expected member and, once a leader exists, leader information under `/<scope>/...`.
- The logs show the startup path the node chose, not just a running PID.

What good looks like:

- a brand new single-node cluster normally settles into a primary-oriented state
- a node joining an existing healthy cluster reports follower-oriented behavior
- fail-safe and trust-related phases are visible through `/ha/state` instead of being hidden behind an API blackout

Common first-run mistakes:

- wrong absolute paths in `process.binaries`
- unreadable password, token, or certificate files
- `pg_hba` rules that do not match the replication or rewind path you configured
- using the wrong etcd scope or endpoints
- PostgreSQL directory permissions that prevent `initdb`, `pg_ctl`, or normal server start

Once those checks are clean, move on to the [Operator Guide](../operator/index.md).
