# Current Tasks Summary

Generated: Sat Mar  7 12:45:47 AM CET 2026

# Task `.ralph/tasks/story-authoritative-managed-postgres-config/04-task-migrate-harness-tests-and-docs-to-the-authoritative-managed-conf-model.md`

```
## Task: Migrate harnesses, tests, and docs to the authoritative managed-conf model <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>01-task-remove-backup-config-and-process-surface,02-task-remove-runtime-restore-bootstrap-and-archive-helper-wiring,04-task-remove-backup-harness-installers-and-gate-selection,05-task-remove-backup-docs-and-obsolete-task-artifacts</blocked_by>
<blocked_by>01-task-introduce-a-typed-managed-postgres-conf-model-and-serializer,02-task-make-pgtm-postgresql-conf-the-only-startup-config-entrypoint,03-task-take-full-ownership-of-replica-recovery-signal-and-auto-conf-state</blocked_by>
```

==============

# Task `.ralph/tasks/story-authoritative-managed-postgres-config/05-task-centralize-composable-sample-runtime-config-builders-for-tests.md`

```
## Task: Centralize composable sample runtime-config builders for tests and helpers <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>01-task-introduce-a-typed-managed-postgres-conf-model-and-serializer,02-task-make-pgtm-postgresql-conf-the-only-startup-config-entrypoint,03-task-take-full-ownership-of-replica-recovery-signal-and-auto-conf-state</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-container-first-deployment/01-task-container-first-docker-deployment-and-compose.md`

```
## Task: Container-first deployment baseline with Docker images, Compose stacks, and secrets <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Make container deployment the default operational path by adding production/development images and turnkey Docker Compose stacks that run etcd3 + pgtuskmaster with config maps and Docker secrets.
```

==============

# Task `.ralph/tasks/story-docs-useful-guides/01-task-rewrite-operator-docs-as-useful-user-guides.md`

```
## Task: Rewrite operator docs as useful user guides and remove horror pages <status>not_started</status> <passes>false</passes>

<description>
Rewrite the non-contributor documentation so it reads like a strong operator/product guide instead of a thin or awkwardly templated book.
```

==============

# Task `.ralph/tasks/story-docs-useful-guides/02-task-rebuild-contributor-docs-as-codebase-navigation-and-contract-guide.md`

```
## Task: Rebuild contributor docs as a codebase navigation and design-contract guide <status>not_started</status> <passes>false</passes>

<description>
Rewrite the contributor documentation so it becomes a genuinely useful guide for understanding the codebase, subsystem boundaries, implementation approach, and design contracts.
```

==============

# Task `.ralph/tasks/story-docs-useful-guides/03-task-align-doc-file-order-and-names-with-rendered-site-structure.md`

```
## Task: Align doc file order and names with the rendered site structure <status>not_started</status> <passes>false</passes>

<description>
Make the docs source tree easier to navigate by aligning file names and ordering conventions with the rendered website structure.
```

==============

# Task `.ralph/tasks/story-docs-useful-guides/04-task-create-repo-readme.md`

```
## Task: Create repository README as the front-door quick-start and project overview <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Add a normal, useful root `README.md` that explains what this project is, how to get started quickly, where to go next for deeper docs, and what the license status is.
```

==============

# Task `.ralph/tasks/story-operator-architecture-docs/04-task-expand-non-contributor-docs-with-deep-subsubchapters.md`

```
## Task: Expand Non-Contributor Docs with Deep Subsubchapters While Keeping Strong Overviews <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Vastly deepen the non-contributor documentation by adding long-form, detail-rich subsubchapters and flowing explanations, while preserving the existing high-level overview quality at chapter entry points.
```

==============

# Task `.ralph/tasks/story-project-wide-code-hygiene/01-task-audit-and-replace-magic-numbers-project-wide.md`

```
## Task: Audit and replace magic numbers project-wide <status>not_started</status> <passes>false</passes> <priority>low</priority>

<description>
Audit the project for unexplained magic numbers and replace them with explicit typed constants, configuration, or otherwise well-justified named values.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/18-task-recurring-meta-deep-skeptical-codebase-review.md`

```
DO NOT PICK THIS TASK UNLESS ALL OTHER TASKS ARE DONE.
## Task: Recurring meta-task for deep skeptical codebase quality verification <status>not_started</status> <passes>meta-task</passes> <priority>very_low</priority>
NEVER TICK OFF THIS TASK. ALWAYS KEEP <passes>meta-task</passes>. This is a recurring deep verification task.

<description>
```

==============

# Task `.ralph/tasks/story-tracing-based-logging/01-task-establish-typed-event-contract-and-emit-ownership-rules.md`

```
## Task: Establish typed event contract and emit ownership rules <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Replace the current partially typed logging contract with a fully typed application event contract that owns event identity, severity, result, and structured fields without requiring call sites to assemble `BTreeMap<String, serde_json::Value>`. The higher order goal is to separate event semantics from backend choice so later decisions about `tracing`, OTEL export, file sinks, or keeping the current sink stack are downstream implementation choices rather than the source of application event truth.
```

==============

# Task `.ralph/tasks/story-tracing-based-logging/02-task-migrate-runtime-process-and-api-to-owned-typed-events.md`

```
## Task: Migrate runtime, process, and api logging to owned typed events <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Convert the most orchestration-heavy logging paths from ad hoc attr maps into typed domain events, and move event ownership to the code that actually owns the semantics of the action or failure. The higher order goal is to stop outer orchestration functions from being the default place where every event is assembled, while still preserving true orchestration boundary events where they add operator value.
```

==============

# Task `.ralph/tasks/story-tracing-based-logging/03-task-migrate-ha-dcs-pginfo-and-postgres-ingest-to-owned-typed-events.md`

```
## Task: Migrate HA, DCS, PgInfo, and Postgres ingest logging to owned typed events <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Convert the remaining control-plane and ingest domains to the typed event contract, with special attention to keeping orchestration decisions separate from operation-owned results and keeping external postgres log lines on a typed raw-record path. The higher order goal is a uniform event model across control-plane state machines and ingest workers, without reintroducing free-form `serde_json` value assembly in domain code.
```

==============

# Task `.ralph/tasks/story-tracing-based-logging/04-task-rework-backends-exporters-tests-and-docs-after-typed-event-migration.md`

```
## Task: Rework logging backends, exporters, tests, and docs after typed event migration <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Revisit backend wiring, exporters, sink abstractions, and documentation only after the typed event contract is in place across the codebase. The higher order goal is to prevent backend work from distorting the event model, and to make any future `tracing` or OTEL integration consume the typed event contract instead of becoming a substitute for it.
```

