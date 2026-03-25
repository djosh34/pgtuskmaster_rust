## Plan: Collapse Command Output Projection Onto NodeState

### Why this is the next reduction target

The CLI command-output boundary is still carrying a shadow projection layer that duplicates seed-state facts instead of letting `NodeState` stay the owner:

- `src/cli/status.rs` fetches `(NodeState, StateQueryOriginDto)` and immediately re-encodes that into `StateCommandOutputDto::from_seed_state(...)`, even though the resulting DTO still stores the full `NodeState` beside the derived projection.
- `src/command/mod.rs` defines `StateProjectionDto`, `StateWarningDto`, `StateHealthDto`, and `StateSwitchoverDto` only to restate data that is already derivable from `NodeState` plus the queried API URL.
- `src/cli/connect.rs` constructs `StateDerivedConnectionCommandDto { projection, targets }` for both `primary` and `replicas`, but `fmt::Display` for that type renders only `targets`, so the non-JSON path computes projection metadata it never reads.
- `tests/ha/support/observer/pgtm.rs` only unwraps `CommandOutputDto::State { output }.state` when observing cluster state, which shows the current JSON command payload is already carrying a removable wrapper around the real owner.

That is boundary duplication, not domain behavior. The real owner is the fetched `NodeState`; human-readable warnings/health rows and any command-specific JSON envelope should derive from that owner directly instead of persisting a second snapshot DTO tree.

### Current overlap already verified

- `src/command/mod.rs:34-115` stores both `projection` and `state` inside `StateCommandOutputDto`, proving the command layer currently keeps duplicated state on the same struct.
- `src/command/mod.rs:52-166` defines six DTO/helper types whose only job is to serialize or format facts already available on `NodeState` and the connection targets.
- `src/cli/status.rs:24-45` exists mostly to bolt `StateQueryOriginDto` onto `NodeState` and feed the duplication layer.
- `src/cli/connect.rs:40-55` and `src/cli/connect.rs:99-103` build a full `StateProjectionDto` for connection commands even though `src/command/mod.rs:192-200` renders only the connection strings.
- `tests/ha/support/observer/pgtm.rs:239-259` consumes only the embedded `NodeState`, not the projection wrapper, when observing status output.

### Execution plan

1. Make `NodeState` the owner of status-command facts.
   - Remove the stored `StateProjectionDto` from the status output path.
   - Keep only the minimal extra command-local facts that are not already on `NodeState`, such as the queried API URL and the verbose flag, and compute warnings/health/switchover summary directly from `NodeState` during rendering or serialization.
   - Reuse existing `NodeState` and `SwitchoverState`; do not introduce another snapshot wrapper if enum fields or private helpers are sufficient.

2. Delete the connection-command projection courier.
   - Remove `projection` from the primary/replicas output path and keep only the connection targets actually rendered by the command.
   - If JSON output still needs a command envelope, let that envelope contain only the concrete connection targets instead of a copied seed-state projection.
   - Keep `StateDerivedConnectionTargetDto` only if it still earns its keep after projection removal; otherwise collapse directly onto the existing connection types.

3. Collapse command rendering onto private helpers over the real owners.
   - Rewrite the status display implementation so warning, health, and switchover lines are derived on demand from `NodeState`.
   - Move any repeated projection logic into narrow private helper functions instead of serializable shadow DTOs.
   - Preserve the existing table semantics and connection-string rendering unless removing them clearly deletes more code without widening the API again.

4. Shrink CLI glue and test support around the collapsed boundary.
   - Simplify `src/cli/status.rs` so it returns the command output directly from the fetched seed state instead of constructing an intermediate projection DTO.
   - Simplify `src/cli/connect.rs` so `primary` and `replicas` stop computing unused projection metadata.
   - Update `tests/ha/support/observer/pgtm.rs` and `src/command/mod.rs` tests to assert against the reduced command payloads and remove fixtures/helpers that only existed to manufacture `StateProjectionDto`.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to stay net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Reuse `NodeState`, `PgConnInfo`, and `SwitchoverState`; do not replace one DTO tree with another equally wide DTO tree.
- Keep JSON output parseable by the in-repo consumers after updating their tests, but do not preserve fields whose only purpose was the duplicated projection boundary.
- Keep human-readable status rendering behavior intact unless a simpler output deletes more code and remains coherent for operators.
- If this starts requiring custom serde machinery that outweighs the DTO removal, switch the plan back to `TO BE VERIFIED`.

### Expected yield

- Delete the `StateProjectionDto` shadow tree or reduce it to a much narrower non-duplicative surface.
- Remove projection construction from `src/cli/connect.rs` and the associated test scaffolding.
- Collapse `src/cli/status.rs` and `src/command/mod.rs` around the real owner (`NodeState`) instead of keeping two snapshots alive.
- Reduce command tests and HA observer plumbing that currently unwrap extra command-output layers only to recover `NodeState`.

NOW EXECUTE
