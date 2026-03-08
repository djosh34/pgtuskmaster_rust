# Why startup planning is separate from steady-state control

Bootstrapping a node requires discovering reality before coordination can begin. The runtime must first interrogate the local data directory, probe the distributed consensus system for cached state, and decide how to align the two. Only after this alignment completes does it make sense to enter the long-running worker topology where assumptions about state can be trusted.

## Startup mode selection

The function `run_node_from_config` orchestrates this bootstrap sequence. After validating configuration and bootstrapping logging, it calls `plan_startup`, which inspects the data directory and DCS cache to select one of three internal modes:

*   `InitializePrimary` – when no viable data directory exists and the node must create a new primary
*   `CloneReplica` – when the node must replicate from an existing primary before joining the topology
*   `ResumeExisting` – when the node discovers intact managed runtime files and must reconstruct its prior role

These modes are runtime classifications, not user-visible commands. The decision emerges from physical evidence, not operator intent.

## Execution before concurrency

Once the mode is selected, `execute_startup` runs a sequential series of actions:

*   Claiming the init lock to prevent concurrent initialization attempts
*   Seeding configuration into the data directory
*   Running any required process-level jobs
*   Starting PostgreSQL with a `ManagedPostgresStartIntent`

The `ResumeExisting` path must additionally call `read_existing_replica_start_intent` to reconstruct the previous managed replica state from disk. Only after all startup actions complete does `run_workers` launch the concurrent worker processes. This ordering guarantees that the data directory and DCS reach a consistent state before any worker can read or modify topology information.

## Operational consequences

Treating startup as a dedicated phase trades runtime simplicity for deterministic sequencing. The init lock and explicit data-dir classification exist because the system cannot safely run the HA state machine while its own existence is uncertain. Worker concurrency assumes a stable substrate; startup builds that substrate. This boundary means operators observe a clear gap between process launch and worker activity, and it ensures that crash-recovery paths do not need to distinguish between partial initialization and full runtime failures.
