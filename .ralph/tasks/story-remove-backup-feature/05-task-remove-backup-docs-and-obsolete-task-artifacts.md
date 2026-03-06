## Task: Remove backup feature docs and delete obsolete pgBackRest task artifacts <status>done</status> <passes>true</passes> <priority>high</priority>

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
- [x] Remove backup/restore/archive/pgBackRest documentation from `docs/src/operator/configuration.md`.
- [x] Remove WAL passthrough observability documentation and `backup.wal_passthrough` references from `docs/src/operator/observability.md`.
- [x] Remove backup/restore troubleshooting guidance from `docs/src/operator/troubleshooting.md`.
- [x] Remove `docs/src/operator/recovery-bootstrap-runbook.md`.
- [x] Remove `docs/src/operator/cluster-restore-takeover-runbook.md`.
- [x] Update `docs/src/SUMMARY.md` and `docs/src/operator/index.md` to drop removed chapters.
- [x] Remove restore and `/events/wal` interface docs from `docs/src/interfaces/node-api.md`.
- [x] Remove backup/pgBackRest mentions from contributor and quick-start docs that are no longer true.
- [x] Regenerate docs so generated `docs/book/` output matches the new source docs.
- [x] Delete the obsolete task files in `.ralph/tasks/story-pgbackrest-managed-backup-recovery/`.
- [x] Delete obsolete Ralph bug/task artifacts that only preserve backup-era feature context:
  - `.ralph/tasks/bugs/unused-backup-recovery-mode-doc-configuration.md`
  - `.ralph/tasks/bugs/logging-archive-ingest-silent-failure-and-unsafe-cleanup.md`
- [x] Confirm by search that `.ralph/tasks/` no longer contains the old pgBackRest story task files.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<execution_plan>
## Detailed Execution Plan (Draft 1, 2026-03-06)

1. Reconfirm the current execution boundary before making doc or task-inventory edits
- Treat the task description as historically useful but partially stale.
- Current HEAD has already drifted in ways execution must respect:
  - `docs/src/operator/recovery-bootstrap-runbook.md` is already absent.
  - `docs/src/operator/cluster-restore-takeover-runbook.md` is already absent.
  - `.ralph/tasks/story-pgbackrest-managed-backup-recovery/` is already absent.
  - `.ralph/tasks/bugs/unused-backup-recovery-mode-doc-configuration.md` is already absent.
  - `.ralph/tasks/bugs/logging-archive-ingest-silent-failure-and-unsafe-cleanup.md` is already absent.
  - `docs/src/SUMMARY.md` already excludes the removed runbook chapters.
- Because of that drift, several acceptance items have become verification work rather than fresh deletion work. Do not recreate stale files or attempt to “fix” references to paths that no longer exist unless a live navigation file still points at them.

2. Product/documentation intent that execution must preserve
- Remove backup-feature documentation, not the surviving replica-clone behavior.
- Keep legitimate `pg_basebackup` references where they describe current replica bootstrap, contributor workflow, or real-binary prerequisites.
- Remove transitional wording that still spends space describing deleted backup surfaces if the same page can now state the surviving behavior directly and more cleanly.
- Keep docs aligned with the current system shape: no repository backup feature, no restore-bootstrap mode, no restore takeover runbook, no WAL passthrough interface or observability surface, and no pgBackRest installer story.
- If execution uncovers unrelated doc rot outside this scope, create a bug with the `add-bug` skill rather than broadening the task silently.

3. Primary source files to inspect or patch during `NOW EXECUTE`
- Operator docs:
  - `docs/src/operator/configuration.md`
  - `docs/src/operator/observability.md`
  - `docs/src/operator/troubleshooting.md`
  - `docs/src/operator/index.md`
- Interface docs:
  - `docs/src/interfaces/node-api.md`
- Contributor docs:
  - `docs/src/contributors/testing-system.md`
  - `docs/src/contributors/harness-internals.md`
  - `docs/src/contributors/codebase-map.md`
- Quick-start docs:
  - `docs/src/quick-start/index.md`
  - `docs/src/quick-start/first-run.md`
  - `docs/src/quick-start/initial-validation.md`
  - `docs/src/quick-start/prerequisites.md`
- Navigation/build outputs:
  - `docs/src/SUMMARY.md`
  - `Makefile` docs targets (`docs-build`, `docs-hygiene`)
- Ralph inventory:
  - `.ralph/tasks/`

4. Execution phase A: clean live doc navigation first so the book stops advertising deleted runbooks
- Update `docs/src/operator/index.md` because it still lists:
  - `Recovery Bootstrap Runbook`
  - `Cluster Restore Takeover Runbook`
- Verify `docs/src/SUMMARY.md` remains consistent with the current chapter set and does not need additional edits.
- Search other `docs/src/` navigation/index pages for stale mentions of the removed runbooks and remove them if found.
- Do not edit generated `docs/book/` directly in this phase; source docs remain the source of truth.

5. Execution phase B: remove backup-feature language from operator and interface docs while preserving real replica-clone docs
- In `docs/src/operator/configuration.md`:
  - remove backup-feature framing that talks about deleted pgBackRest/backup configuration surfaces unless that sentence is necessary to explain the current schema
  - keep the live `pg_basebackup`, rewind-source, and replica bootstrap guidance
  - ensure no `restore-bootstrap`, WAL archive helper, `archive_command`, or `restore_command` guidance remains except possibly as negative guardrails if they are still necessary and clearly about unsupported settings
- In `docs/src/operator/observability.md`:
  - remove any WAL passthrough observability text and any `backup.wal_passthrough` references
- In `docs/src/operator/troubleshooting.md`:
  - remove backup/restore troubleshooting instructions that describe deleted feature paths
  - preserve troubleshooting for surviving startup, rewind, and replica-clone flows
- In `docs/src/interfaces/node-api.md`:
  - remove restore and `/events/wal` interface documentation if any remains
  - if the page already has a short negative statement that those APIs do not exist, simplify it so the interface chapter describes only live APIs

6. Execution phase C: clean contributor and quick-start docs of deleted-feature context
- Search and edit only where the docs still preserve backup-era context:
  - `docs/src/contributors/testing-system.md`
  - `docs/src/contributors/harness-internals.md`
  - `docs/src/contributors/codebase-map.md`
  - quick-start pages under `docs/src/quick-start/`
- Preserve legitimate references to:
  - `pg_basebackup`
  - rewind
  - real PostgreSQL binaries required for the surviving system
- Remove mentions of:
  - pgBackRest installation
  - restore takeover
  - restore bootstrap
  - WAL passthrough observability or events
  - backup config blocks that no longer exist
- If a page only contains valid `pg_basebackup` content and no deleted-feature residue, leave it unchanged.

7. Execution phase D: verify Ralph inventory cleanup and only delete live obsolete artifacts
- Search `.ralph/tasks/` for:
  - `pgbackrest`
  - `backup`
  - `restore`
  - `wal`
- Interpret hits carefully:
  - the active `story-remove-backup-feature` tasks are expected and must remain
  - historical references inside unrelated active tasks do not automatically qualify for deletion unless they are obsolete artifacts that only preserve the removed feature story
- Confirm the old pgBackRest story directory is absent rather than trying to delete it again.
- Confirm the two named bug files are absent rather than trying to recreate/delete them.
- If any other clearly obsolete backup-only Ralph artifacts still exist and are not part of the active removal story, delete them in this task and mention the evidence in the task file before ticking acceptance boxes.

8. Execution phase E: validate the docs build using the repo’s docs targets, without treating generated output as tracked work
- `docs/book/` is generated/ignored output in this repo. Do not plan around committing or reviewing a `docs/book/` git diff.
- Run `make docs-build` after source-doc edits so the book renders successfully from the updated navigation and chapters.
- Run `make docs-hygiene` to verify generated docs output is not tracked in git.
- If useful, spot-check the locally rendered `docs/book/` output after the build, but treat that as verification only; the source-of-truth edits remain under `docs/src/`.

9. Search checklist that must be run during execution
- Use tracked-source searches first, excluding generated noise where appropriate.
- Required search terms across `docs/src/` and `.ralph/tasks/`:
  - `pgBackRest`
  - `pgbackrest`
  - `wal_passthrough`
  - `/events/wal`
  - `restore-bootstrap`
  - `restore bootstrap`
  - `restore takeover`
  - `archive_command`
  - `restore_command`
  - `backup.`
- Interpret results carefully:
  - `pg_basebackup` hits are expected and usually must remain
  - negative assertions in docs that merely say a deleted API does not exist should be removed if the page can simply describe the live surface without transitional language
  - hits inside generated `docs/book/` before the rebuild are expected and should not drive direct edits

10. Checkbox and task-file update order for the later `NOW EXECUTE` pass
1. Clean source docs and navigation.
2. Regenerate docs output.
3. Re-run searches and confirm obsolete Ralph artifacts are absent or deleted.
4. Tick acceptance boxes only after evidence exists for each one.
5. Run docs validation before the full repo gates:
   - `make docs-build`
   - `make docs-hygiene`
6. Run the required full gates in this order:
   - `make check`
   - `make test`
   - `make test-long`
   - `make lint`
7. Only after every gate passes, set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit all changes including `.ralph`, and push.

11. Constraints to honor during execution
- Do not do fresh broad exploration once the task reaches `NOW EXECUTE`; execute this sequence directly.
- Do not remove legitimate replica-clone documentation just because it contains the word `basebackup`.
- Do not skip `make test-long`; the user completion rule explicitly requires it.
- Keep edits ASCII and lint-clean, with no swallowed errors, `unwrap`, `expect`, or panic-introducing changes if any code or tooling scripts must be touched.
</execution_plan>

NOW EXECUTE
