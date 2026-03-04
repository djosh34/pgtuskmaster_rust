---
## Task: Rust WAL Passthrough Binary for Postgres Archive Restore Logging <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Reintroduce archive/restore observability with a Rust binary command invoked by Postgres that performs passthrough execution and logs invocations via pgtuskmaster.

**Scope:**
- Add a generic Rust passthrough executable command surface dedicated to Postgres archive/restore invocation passthrough with strict exit-code fidelity.
- Keep Postgres config model with explicit `archive_command` / `restore_command` strings, but route commands to the Rust binary instead of shell wrapper generation, with no shell parsing.
- Implement logging flow where passthrough process sends structured event payloads to pgtuskmaster (API or equivalent local interface) and pgtuskmaster emits normal structured logs.
- Support both operation kinds (`archive-push`, `archive-get`) and preserve full command argument semantics.
- Enforce argv-only execution model: no command-string parsing, no shell eval, no quoting reconstruction; child processes are spawned from already-tokenized args.
- Enforce absolute-path-only executable configuration for passthrough target commands and `pgbackrest`; do not perform PATH lookup or fallback resolution.

**Context from research:**
- Existing shell wrapper logs by writing JSONL directly to file and returning pgBackRest exit code.
- Desired architecture: no generated scripts, no shell JSON construction, and no wrapper-specific file format contract.
- Existing config and managed postgres wiring already controls archive/restore command assignment and can be adapted to point at Rust binary command lines.

**Expected outcome:**
- Postgres calls a Rust binary command for archive and restore actions.
- Rust passthrough executes target backup command, forwards/stores output safely, and returns exact exit status.
- Each invocation is logged through pgtuskmaster with structured fields equivalent or better than current backup event model.
- No bash or shell-wrapper code path remains.
- Command execution is fully deterministic from configured argv vectors with absolute executable paths only.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Add Rust binary/subcommand implementation (for example under [src/bin/](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/bin)) that supports explicit modes equivalent to archive push/get and validates required args.
- [ ] Implement strict passthrough behavior: execute configured backup command/toolchain from tokenized argv vectors, capture bounded output, and return exact child exit code to caller.
- [ ] Passthrough command execution must be argv-native only (`std::process::Command` style argument vector usage): no shell invocation, no command-string parsing, and no re-tokenization logic.
- [ ] Implement structured event emission from passthrough binary to pgtuskmaster logging surface (API or internal endpoint), with robust error handling that never masks underlying backup command exit semantics.
- [ ] Update configuration schema/defaults and managed-postgres wiring in [src/config/schema.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config/schema.rs), [src/config/defaults.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config/defaults.rs), [src/config/parser.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config/parser.rs), and [src/postgres_managed.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs) so Postgres command strings target the Rust passthrough binary.
- [ ] Config surface for passthrough target commands must be array-of-args only (no free-form command strings); executable path must be absolute.
- [ ] Config validation must reject non-absolute executable paths and must reject PATH-based command resolution for passthrough targets and `process.binaries.pgbackrest`.
- [ ] Align `pgbackrest` handling with other binaries: mandatory explicit configured absolute path, never PATH lookup, never implicit command-name execution.
- [ ] Preserve or improve existing backup event fields (`provider`, `event_kind`, `invocation_id`, `status_code`, `success`, output truncation indicator, and relevant WAL path fields) in centralized pgtuskmaster logs.
- [ ] Add integration tests covering:
- [ ] Postgres archive push path invokes Rust binary and returns child success.
- [ ] Postgres archive get path invokes Rust binary and returns child failure exactly.
- [ ] Structured log events are emitted through pgtuskmaster with expected fields under concurrent invocations.
- [ ] Special-character/space/single-quote argument paths are handled correctly through argv vectors without shell-wrapper quoting hacks.
- [ ] Tests prove non-absolute binary paths are rejected during config validation and PATH is never consulted.
- [ ] Update docs to describe Rust passthrough architecture and remove obsolete wrapper-file guidance in [docs/src/operator/observability.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/operator/observability.md), [docs/src/operator/configuration.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/operator/configuration.md), and relevant lifecycle docs.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
