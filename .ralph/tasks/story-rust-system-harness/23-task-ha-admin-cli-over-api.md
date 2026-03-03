---
## Task: Build a simple Rust HA admin CLI over the exposed API <status>done</status> <passes>true</passes> <passing>true</passing>

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
- [x] Full exhaustive checklist completed with concrete module requirements: `Cargo.toml` (CLI deps), `src/bin/pgtuskmasterctl.rs` (command tree), `src/api/*` or new client module (request/response client), `tests/` CLI coverage (parse + transport mocks), docs/readme command examples
- [x] `make check` — passes cleanly
- [x] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail) (`no marker found` in output; command exit 0)
- [x] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail) (`no marker found` in output; command exit 0)
- [x] `make test-bdd` — all BDD features pass
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2, Skeptically Verified)

### Deep skeptical verification snapshot (completed, 16+ tracks)
- Re-checked API contract and route auth behavior from source, not assumptions:
  - routes exist exactly as `GET /ha/state`, `POST /ha/leader`, `DELETE /ha/leader`, `DELETE /ha/switchover`, `POST /switchover`.
  - auth policy allows admin token on all routes and read token only on read routes.
- Verified `src/api/mod.rs` response structs are `pub(crate)`, so external reuse from new CLI modules would force unnecessary visibility widening.
- Verified there is no existing binary target in `src/bin/`.
- Verified strict lint denies unwrap/expect/panic/todo/unimplemented globally.
- Verified Make targets required by this task are exactly `check`, `test`, `test-bdd`, `lint`.
- Verified docs baseline: no dedicated CLI doc exists yet.

### Verification delta (mandatory plan changes)
1. **Changed API type strategy**:
- Old plan: promote/reuse API module structs from `src/api/mod.rs`.
- New plan: keep API internals untouched and define CLI-local DTOs for request/response payloads.
- Rationale: avoids broadening crate API surface just to satisfy CLI client code and reduces regression risk in existing API module boundaries.

2. **Changed binary smoke test strategy**:
- Old plan: add `assert_cmd` + `predicates`.
- New plan: implement binary invocation checks with `std::process::Command` in integration tests to avoid introducing extra dev-dependencies unless needed.
- Rationale: simpler dependency graph and less compile churn while still validating exit code/stdout/stderr behavior.

### Architecture decision for CLI
1. CLI parser and UX:
- Use `clap` derive API.
- Command root: `pgtuskmasterctl`.
- Global flags:
  - `--base-url <URL>` (default `http://127.0.0.1:8008`)
  - `--read-token <TOKEN>` (optional; env `PGTUSKMASTER_READ_TOKEN`)
  - `--admin-token <TOKEN>` (optional; env `PGTUSKMASTER_ADMIN_TOKEN`)
  - `--timeout-ms <MILLIS>` (default 5000)
  - `--output <json|text>` (default `json`)

2. Command tree:
- `ha state`
- `ha leader set --member-id <ID>`
- `ha leader clear`
- `ha switchover clear`
- `ha switchover request --requested-by <ID>` (calls legacy-compatible `POST /switchover`)

3. Client strategy:
- Add async HTTP client via `reqwest` + `rustls-tls`.
- Keep all CLI HTTP logic in a dedicated module and keep route payload DTOs CLI-local.
- Token selection policy:
  - `ha state` uses read token when present, else admin token.
  - admin-write commands use admin token.
  - if selected token is absent, send no Authorization header (to preserve compatibility with tokenless server config).

4. Exit code/error mapping:
- `0` success.
- `2` usage/arg parse (clap).
- `3` transport/timeout.
- `4` non-2xx API status.
- `5` decode/schema mismatch.

### Module-level implementation plan
1. `Cargo.toml`
- Add dependencies:
  - `clap = { version = "4", features = ["derive", "env"] }`
  - `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }`

2. Binary entrypoint
- Add `src/bin/pgtuskmasterctl.rs` with `#[tokio::main]`.
- Parse args via clap and delegate to library runner.
- Map runner result to `std::process::ExitCode` without unwrap/panic.

3. New library CLI module
- Add `src/cli/mod.rs`, `src/cli/args.rs`, `src/cli/client.rs`, `src/cli/error.rs`, `src/cli/output.rs`.
- Export from `src/lib.rs` as `pub mod cli;`.
- Request methods:
  - `get_ha_state()`
  - `post_set_leader(member_id)`
  - `delete_leader()`
  - `delete_switchover()`
  - `post_switchover(requested_by)`

4. CLI-local DTOs
- `AcceptedResponse` and `HaStateResponse` in CLI client/output module with `serde(deny_unknown_fields)` for stricter contract checks.
- `SetLeaderRequestInput` and `SwitchoverRequestInput` local to CLI client for POST bodies.

5. Documentation
- Add CLI usage section with concrete command examples and token/env usage to `RUST_SYSTEM_HARNESS_PLAN.md` (or dedicated CLI doc if that proves cleaner during implementation).

### Test plan (no optional skips)
1. Parsing tests:
- command tree parse success/failure, required args, defaults/env behavior.

2. HTTP request-construction tests:
- ephemeral local TCP listener validates method/path/header/body for each command.
- validate read/admin token header selection.

3. Error mapping tests:
- non-2xx maps to exit code `4`.
- malformed JSON on 2xx maps to `5`.
- connection refused/timeout maps to `3`.

4. Binary invocation tests:
- `pgtuskmasterctl --help` and representative command run using `std::process::Command`.
- verify exit code and stdout/stderr split.

5. Required gates:
- `make check`
- `make test`
- `make test-bdd`
- `make lint`

### Execution sequence
1. Add deps and CLI module/binary scaffolding.
2. Implement parser, client, output, and error mapping.
3. Implement parsing/request/error/binary tests.
4. Add docs examples.
5. Run all required gates and only then update task pass markers/checklist.
</execution_plan>

NOW EXECUTE
