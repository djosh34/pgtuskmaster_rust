## Plan: Delete Command Output Envelope

### Why this is the next reduction target

The previous slice collapsed the inner projection DTO tree, but the CLI boundary still keeps a top-level command envelope that in-repo callers immediately unwrap again:

- `src/command/mod.rs` now exists almost entirely to define `CommandOutputDto`, `fmt::Display`, and a handful of rendering helpers.
- `src/cli/status.rs` still boxes `NodeState` into `CommandOutputDto::State` even though the only extra facts are `api_url` and `verbose`, which are formatting inputs rather than owned command data.
- `src/cli/connect.rs` wraps `Vec<PgConnInfo>` in `CommandOutputDto::{Primary,Replicas}` only to serialize or print the same connection strings.
- `src/cli/switchover.rs` wraps `AcceptedResponse` in `CommandOutputDto::Switchover`, and `src/command/mod.rs` still carries a `ReloadCertificates` variant that no CLI entrypoint constructs at all.
- `tests/ha/support/observer/pgtm.rs` parses `CommandOutputDto` only to recover `NodeState` or `AcceptedResponse`, which proves the enum is now a courier layer rather than a domain owner.

That means the command module is no longer modeling behavior. It is a tagged transport wrapper around already-owned values.

### Current overlap already verified

- `rg -n "CommandOutputDto" src tests` shows references only in `src/cli/status.rs`, `src/cli/connect.rs`, `src/cli/switchover.rs`, `src/command/mod.rs`, and `tests/ha/support/observer/pgtm.rs`.
- `src/lib.rs` publicly exports `command`, so the wrapper survives today as API surface rather than necessity.
- `src/cli/status.rs:16-23` and `src/cli/status.rs:37-44` fetch `(NodeState, api_url)` and immediately rewrap them.
- `src/cli/connect.rs:34-46` and `src/cli/connect.rs:77-87` rewrap direct `Vec<PgConnInfo>` targets without adding behavior.
- `src/cli/switchover.rs:14-19` and `src/cli/switchover.rs:29-33` rewrap `AcceptedResponse` without adding behavior.
- `tests/ha/support/observer/pgtm.rs:289-299` deserializes `CommandOutputDto`, then `tests/ha/support/observer/pgtm.rs:252-259` and `tests/ha/support/observer/pgtm.rs:220-234` immediately unwrap the real owners.
- `rg -n "ReloadCertificatesResponse|ReloadCertificates" src tests` shows the reload-certificates response is produced directly by the API worker, while the command-envelope variant is only defined in `src/command/mod.rs` and mentioned in the observer’s label helper.

### Execution plan

1. Collapse command serialization onto the real owners.
   - Remove `CommandOutputDto` from the CLI status, connect, and switchover paths.
   - Emit JSON directly from `NodeState`, `Vec<PgConnInfo>`, and `AcceptedResponse`.
   - Keep non-JSON rendering via plain helper functions instead of a tagged enum wrapper.

2. Move display-only helpers out of the courier module.
   - Keep the status table formatting logic, but make it operate on `NodeState`, `api_url`, and `verbose` directly.
   - Keep connection-string rendering on `&[PgConnInfo]` directly.
   - If there is no remaining non-test reason for `src/command/mod.rs` to exist, delete the module and stop exporting it from `src/lib.rs`.

3. Shrink HA observer plumbing onto direct parses.
   - Parse `pgtm status --json` as `NodeState`.
   - Parse `pgtm switchover request --json` as `AcceptedResponse`.
   - Remove the command-label matcher and any failure text that only exists because of the enum envelope.

4. Update tests around the reduced boundary.
   - Move or rewrite the current rendering tests from `src/command/mod.rs` onto the new helper location.
   - Update HA observer tests to assert the direct JSON payloads.
   - Remove assertions that depend on the `"command"` discriminator field.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to stay net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Reuse `NodeState`, `PgConnInfo`, and `AcceptedResponse`; do not replace `CommandOutputDto` with another equally wide wrapper enum or DTO tree.
- Keep the human-readable status table semantics intact unless a simpler render path clearly deletes more code without reducing clarity.
- Keep the in-repo JSON consumers parseable after their tests are updated, but do not preserve the `"command"` tag just to keep the dead envelope alive.
- If deleting `src/command/mod.rs` uncovers a real shared abstraction that is still earning its keep, narrow it to render helpers only and avoid recreating the transport wrapper.

### Expected yield

- Delete most or all of `src/command/mod.rs`, including the dead reload-certificates variant and the command-envelope serde tests.
- Shrink three CLI entrypoints by returning/rendering their real owners directly.
- Simplify `tests/ha/support/observer/pgtm.rs` by parsing direct payloads instead of tagged command wrappers.
- Remove one more public-but-unnecessary boundary from `src/lib.rs`.

NOW EXECUTE
