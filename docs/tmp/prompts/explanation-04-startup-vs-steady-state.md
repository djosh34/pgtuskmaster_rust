Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Draft a new explanation page.

[Output path]
- docs/src/explanation/startup-versus-steady-state.md

[Page title]
- # Why startup planning is separate from steady-state control

[Audience]
- Engineers trying to understand why node startup is handled as a special phase before the normal worker loop starts.

[User need]
- Understand why the runtime inspects the data directory, probes DCS, selects a startup mode, executes startup actions, and only then enters the long-running worker topology.

[mdBook context]
- Link naturally to node runtime, managed PostgreSQL runtime files, and HA state machine reference pages.

[Diataxis guidance]
- Explanation only.
- Center the reasoning, not the startup action catalog.

[Verified facts that are true]
- run_node_from_config validates runtime config, bootstraps logging, plans startup, executes startup, and then runs workers.
- Startup planning inspects data directory state, probes DCS cache, and selects one of InitializePrimary, CloneReplica, or ResumeExisting.
- Startup actions include claiming the init lock and seeding config, running process jobs, and starting Postgres with a ManagedPostgresStartIntent.
- The runtime reconstructs resume intent from existing managed replica state when needed.
- Worker startup occurs only after startup execution has completed.

[Relevant repo grounding]
- src/runtime/node.rs: run_node_from_config, plan_startup, StartupMode, StartupAction, execute_startup, run_workers.
- src/postgres_managed.rs: read_existing_replica_start_intent and managed runtime file handling.

[Design tensions to explain]
- Why startup cannot be treated as just another HA tick.
- Why the system needs an explicit init lock and data-dir classification.
- Why resume-existing is distinct from clone and initialize modes.
- The tradeoff between deterministic startup sequencing and runtime simplicity.

[Required structure]
- Open with the difference between bootstrapping reality and steady-state coordination.
- Explain the startup mode selection idea.
- Explain why startup is resolved before worker concurrency begins.
- Close with the operational consequences of this boundary.

[Facts that must not be invented or changed]
- Do not claim HA manages startup from the first instruction of the process.
- Do not claim startup modes are user-visible commands.
