---
## Task: Build a simple Rust HA admin CLI over the exposed API <status>not_started</status> <passes>false</passes>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>

<description>
**Goal:** Provide a simple, production-usable Rust CLI that invokes the HA admin API for both read and write operations.

**Scope:**
- Add a CLI binary entrypoint (for example `src/bin/pgtuskmasterctl.rs`) with subcommands for cluster reads and HA control writes.
- Use an established Rust CLI crate (prefer `clap`) for argument parsing and help UX.
- Implement API client calls using existing async/runtime stack (`tokio`, HTTP request path already used in tests).
- Add CLI-focused tests (argument parsing, request construction, and error mapping) and docs for command usage.

**Context from research:**
- There is currently no runtime/admin CLI in the repository.
- The requested workflow is API-driven HA administration, not direct DCS/binary control.
- CLI should match the new API contracts and become the supported operator surface for e2e orchestration.

**Expected outcome:**
- A small `pgtuskmasterctl` command can read cluster/HA status and trigger admin actions through API endpoints with clear exit codes and errors.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: `Cargo.toml` (CLI deps), `src/bin/pgtuskmasterctl.rs` (command tree), `src/api/*` or new client module (request/response client), `tests/` CLI coverage (parse + transport mocks), docs/readme command examples
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
