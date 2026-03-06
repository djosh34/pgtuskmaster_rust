# Current Tasks Summary

Generated: Fri Mar  6 13:29:30 CET 2026

**Path:** `.ralph/tasks/bugs/ha-action-deduping-suppresses-retry.md`

---
## Bug: HA action dedupe suppresses legitimate retries <status>blocked</status> <passes>false</passes>

<blocked_by>06-task-move-and-split-ha-e2e-tests-after-functional-rewrite</blocked_by>

---

**Path:** `.ralph/tasks/bugs/ha-decide-mutation-heavy-control-flow-needs-pure-refactor.md`

---
## Bug: HA decide mutation-heavy control flow needs pure refactor <status>blocked</status> <passes>false</passes>

<blocked_by>06-task-move-and-split-ha-e2e-tests-after-functional-rewrite</blocked_by>

---

**Path:** `.ralph/tasks/bugs/restore-terminal-phases-keep-ha-fencing.md`

---
## Bug: Restore terminal phases keep HA in repeated fencing <status>blocked</status> <passes>false</passes>

<blocked_by>05-task-remove-backup-docs-and-obsolete-task-artifacts</blocked_by>
<blocked_by>06-task-move-and-split-ha-e2e-tests-after-functional-rewrite</blocked_by>

---

**Path:** `.ralph/tasks/story-container-first-deployment/01-task-container-first-docker-deployment-and-compose.md`

---
## Task: Container-first deployment baseline with Docker images, Compose stacks, and secrets <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Make container deployment the default operational path by adding production/development images and turnkey Docker Compose stacks that run etcd3 + pgtuskmaster with config maps and Docker secrets.

---

**Path:** `.ralph/tasks/story-docs-useful-guides/01-task-rewrite-operator-docs-as-useful-user-guides.md`

---
## Task: Rewrite operator docs as useful user guides and remove horror pages <status>not_started</status> <passes>false</passes>

<description>
Rewrite the non-contributor documentation so it reads like a strong operator/product guide instead of a thin or awkwardly templated book.

---

**Path:** `.ralph/tasks/story-docs-useful-guides/02-task-rebuild-contributor-docs-as-codebase-navigation-and-contract-guide.md`

---
## Task: Rebuild contributor docs as a codebase navigation and design-contract guide <status>not_started</status> <passes>false</passes>

<description>
Rewrite the contributor documentation so it becomes a genuinely useful guide for understanding the codebase, subsystem boundaries, implementation approach, and design contracts.

---

**Path:** `.ralph/tasks/story-docs-useful-guides/03-task-align-doc-file-order-and-names-with-rendered-site-structure.md`

---
## Task: Align doc file order and names with the rendered site structure <status>not_started</status> <passes>false</passes>

<description>
Make the docs source tree easier to navigate by aligning file names and ordering conventions with the rendered website structure.

---

**Path:** `.ralph/tasks/story-docs-useful-guides/04-task-create-repo-readme.md`

---
## Task: Create repository README as the front-door quick-start and project overview <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Add a normal, useful root `README.md` that explains what this project is, how to get started quickly, where to go next for deeper docs, and what the license status is.

---

**Path:** `.ralph/tasks/story-greenfield-secure-config/01-task-remove-config-versioning-and-restore-a-greenfield-config-contract.md`

---
## Task: Remove config versioning and restore a greenfield config contract <status>not_started</status> <passes>false</passes>

<description>
Remove user-facing config versioning from the product and restore a simple greenfield config contract with no fake `v2` framing.

---

**Path:** `.ralph/tasks/story-greenfield-secure-config/02-task-simplify-config-semantics-and-make-secure-mtls-the-documented-default.md`

---
## Task: Simplify config semantics and make secure mTLS the documented default <status>not_started</status> <passes>false</passes>

<description>
Rework the config contract and documentation so the supported settings make operational sense and the recommended setup is secure by default.

---

**Path:** `.ralph/tasks/story-greenfield-secure-config/03-task-derive-rewind-source-from-current-primary-instead-of-static-config.md`

---
## Task: Derive rewind source from the current primary instead of static config <status>not_started</status> <passes>false</passes>

<description>
Remove static rewind source addressing from the product and derive rewind behavior from current cluster state.

---

**Path:** `.ralph/tasks/story-operator-architecture-docs/04-task-expand-non-contributor-docs-with-deep-subsubchapters.md`

---
## Task: Expand Non-Contributor Docs with Deep Subsubchapters While Keeping Strong Overviews <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Vastly deepen the non-contributor documentation by adding long-form, detail-rich subsubchapters and flowing explanations, while preserving the existing high-level overview quality at chapter entry points.

---

**Path:** `.ralph/tasks/story-project-wide-code-hygiene/01-task-audit-and-replace-magic-numbers-project-wide.md`

---
## Task: Audit and replace magic numbers project-wide <status>not_started</status> <passes>false</passes> <priority>low</priority>

<description>
Audit the project for unexplained magic numbers and replace them with explicit typed constants, configuration, or otherwise well-justified named values.

---

**Path:** `.ralph/tasks/story-remove-backup-feature/01-task-remove-backup-config-and-process-surface.md`

---
## Task: Remove backup config and pgBackRest process vocabulary while keeping basebackup replica cloning <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Delete the backup feature's config and process-language surface completely, while preserving `pg_basebackup`-based replica creation as a non-backup bootstrap path.

---

**Path:** `.ralph/tasks/story-remove-backup-feature/02-task-remove-runtime-restore-bootstrap-and-archive-helper-wiring.md`

---
## Task: Remove runtime restore bootstrap and the archive_command helper/proxy wiring <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Delete the runtime-owned restore bootstrap path and the hacky archive/restore helper stack, including the local event-ingest API used only for archive_command/restore_command passthrough logging.

---

**Path:** `.ralph/tasks/story-remove-backup-feature/04-task-remove-backup-harness-installers-and-gate-selection.md`

---
## Task: Remove backup-specific harness, installer, and gate-selection surfaces while preserving real tests for replica cloning <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Delete the backup feature's harness and packaging residue so real-binary verification no longer provisions or expects pgBackRest, while preserving real coverage for normal Postgres and replica-clone behavior.

---

**Path:** `.ralph/tasks/story-remove-backup-feature/05-task-remove-backup-docs-and-obsolete-task-artifacts.md`

---
## Task: Remove backup feature docs and delete obsolete pgBackRest task artifacts <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Remove all operator/interface/contributor documentation for the backup feature and clean the Ralph task inventory so it no longer contains implementation tasks for a feature we are deliberately deleting.

---

**Path:** `.ralph/tasks/story-rust-system-harness/18-task-recurring-meta-deep-skeptical-codebase-review.md`

DO NOT PICK THIS TASK UNLESS ALL OTHER TASKS ARE DONE.
## Task: Recurring meta-task for deep skeptical codebase quality verification <status>not_started</status> <passes>meta-task</passes> <priority>very_low</priority>
NEVER TICK OFF THIS TASK. ALWAYS KEEP <passes>meta-task</passes>. This is a recurring deep verification task.

<description>

---

**Path:** `.ralph/tasks/story-tracing-based-logging/01-task-establish-typed-event-contract-and-emit-ownership-rules.md`

---
## Task: Establish typed event contract and emit ownership rules <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Replace the current partially typed logging contract with a fully typed application event contract that owns event identity, severity, result, and structured fields without requiring call sites to assemble `BTreeMap<String, serde_json::Value>`. The higher order goal is to separate event semantics from backend choice so later decisions about `tracing`, OTEL export, file sinks, or keeping the current sink stack are downstream implementation choices rather than the source of application event truth.

---

**Path:** `.ralph/tasks/story-tracing-based-logging/02-task-migrate-runtime-process-and-api-to-owned-typed-events.md`

---
## Task: Migrate runtime, process, and api logging to owned typed events <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Convert the most orchestration-heavy logging paths from ad hoc attr maps into typed domain events, and move event ownership to the code that actually owns the semantics of the action or failure. The higher order goal is to stop outer orchestration functions from being the default place where every event is assembled, while still preserving true orchestration boundary events where they add operator value.

---

**Path:** `.ralph/tasks/story-tracing-based-logging/03-task-migrate-ha-dcs-pginfo-and-postgres-ingest-to-owned-typed-events.md`

---
## Task: Migrate HA, DCS, PgInfo, and Postgres ingest logging to owned typed events <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Convert the remaining control-plane and ingest domains to the typed event contract, with special attention to keeping orchestration decisions separate from operation-owned results and keeping external postgres log lines on a typed raw-record path. The higher order goal is a uniform event model across control-plane state machines and ingest workers, without reintroducing free-form `serde_json` value assembly in domain code.

---

**Path:** `.ralph/tasks/story-tracing-based-logging/04-task-rework-backends-exporters-tests-and-docs-after-typed-event-migration.md`

---
## Task: Rework logging backends, exporters, tests, and docs after typed event migration <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Revisit backend wiring, exporters, sink abstractions, and documentation only after the typed event contract is in place across the codebase. The higher order goal is to prevent backend work from distorting the event model, and to make any future `tracing` or OTEL integration consume the typed event contract instead of becoming a substitute for it.

