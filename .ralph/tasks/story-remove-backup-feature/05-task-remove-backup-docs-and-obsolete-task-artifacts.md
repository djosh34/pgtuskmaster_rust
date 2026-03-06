---
## Task: Remove backup feature docs and delete obsolete pgBackRest task artifacts <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Remove all operator/interface/contributor documentation for the backup feature and clean the Ralph task inventory so it no longer contains implementation tasks for a feature we are deliberately deleting.
This documentation and inventory cleanup belongs to the same high-priority removal story so the repo stops advertising or preserving a feature we have already decided to delete.

**Scope:**
- Delete or rewrite docs that describe backup config, restore bootstrap, restore takeover, WAL passthrough observability, and pgBackRest installation.
- Remove the obsolete pgBackRest story task files after the new removal story exists.
- Remove obsolete Ralph bug/task artifacts that only preserve backup-era context we have decided to delete.

**Context from research:**
- Current documentation mentions the feature in `docs/src/operator/configuration.md`, `docs/src/operator/observability.md`, `docs/src/operator/troubleshooting.md`, `docs/src/operator/recovery-bootstrap-runbook.md`, `docs/src/operator/cluster-restore-takeover-runbook.md`, `docs/src/interfaces/node-api.md`, `docs/src/contributors/testing-system.md`, `docs/src/contributors/harness-internals.md`, `docs/src/contributors/codebase-map.md`, and quick-start pages.
- Generated book output under `docs/book/` will need regeneration after source-doc cleanup.
- Obsolete implementation tasks currently live in `.ralph/tasks/story-pgbackrest-managed-backup-recovery/`.
- Known obsolete Ralph bug/task artifacts that should not survive this story include:
  - `.ralph/tasks/bugs/unused-backup-recovery-mode-doc-configuration.md`
  - `.ralph/tasks/bugs/logging-archive-ingest-silent-failure-and-unsafe-cleanup.md`

**Expected outcome:**
- Public and contributor docs no longer claim the system supports repository backup, restore takeover, WAL passthrough helper logging, or pgBackRest installation.
- Ralph task inventory no longer contains the old pgBackRest implementation story.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Remove backup/restore/archive/pgBackRest documentation from `docs/src/operator/configuration.md`.
- [ ] Remove WAL passthrough observability documentation and `backup.wal_passthrough` references from `docs/src/operator/observability.md`.
- [ ] Remove backup/restore troubleshooting guidance from `docs/src/operator/troubleshooting.md`.
- [ ] Remove `docs/src/operator/recovery-bootstrap-runbook.md`.
- [ ] Remove `docs/src/operator/cluster-restore-takeover-runbook.md`.
- [ ] Update `docs/src/SUMMARY.md` and `docs/src/operator/index.md` to drop removed chapters.
- [ ] Remove restore and `/events/wal` interface docs from `docs/src/interfaces/node-api.md`.
- [ ] Remove backup/pgBackRest mentions from contributor and quick-start docs that are no longer true.
- [ ] Regenerate docs so generated `docs/book/` output matches the new source docs.
- [ ] Delete the obsolete task files in `.ralph/tasks/story-pgbackrest-managed-backup-recovery/`.
- [ ] Delete obsolete Ralph bug/task artifacts that only preserve backup-era feature context:
  - `.ralph/tasks/bugs/unused-backup-recovery-mode-doc-configuration.md`
  - `.ralph/tasks/bugs/logging-archive-ingest-silent-failure-and-unsafe-cleanup.md`
- [ ] Confirm by search that `.ralph/tasks/` no longer contains the old pgBackRest story task files.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
