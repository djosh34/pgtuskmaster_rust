## Plan: Move Fault Marker File Ownership Into Faults Owner

### Why this reduction target

The next wrong-place hotspot is the fault-marker file path and file-write block in `tests/ha/support/world/mod.rs`:

- `tests/ha/support/world/mod.rs` is still 1349 lines and still owns `host_fault_dir`, `host_fault_marker_path`, `write_fault_marker`, and `remove_fault_marker`.
- `tests/ha/support/faults/mod.rs` already owns `FAULT_DIR`, all blocker marker path constants, and the fault-plumbing script contract. The remaining “map a fault marker path into a materialized member directory and write/remove the marker file” behavior belongs with that owner, not with the world/harness bootstrap file.
- The current split forces `world/mod.rs` to understand the static fault directory layout even though the `faults` owner already defines the fault root and marker paths, which is the same wrong-place smell as the previous given-fixture seam.

### Current overlap already verified

- `tests/ha/support/faults/mod.rs` already defines `FAULT_DIR`, `BlockerKind::marker_path`, `BlockerKind::clear_on_start_marker_path`, and all shell scripts that rely on that layout.
- `tests/ha/support/world/mod.rs` only uses the marker-file helpers to toggle blocker and wipe-data marker files under the materialized run directory.
- The marker helpers in `world/mod.rs` do not depend on docker state; they only need the materialized root, member id, and marker path, which makes them a clean fit for the `faults` owner.

### Execution plan

1. Move marker-file path ownership into `tests/ha/support/faults/mod.rs`.
   - Add faults-owned helpers that map a marker path under `FAULT_DIR` into a member-specific materialized fault directory and write/remove the marker file.
   - Reuse the shared `tests/ha/support/files.rs` helpers instead of recreating generic file-writing wrappers.

2. Narrow `world/mod.rs` to orchestration.
   - Replace `host_fault_dir`, `host_fault_marker_path`, `write_fault_marker`, and `remove_fault_marker` in `tests/ha/support/world/mod.rs` with direct calls into the faults owner.
   - Keep `world/mod.rs` responsible only for deciding when to toggle blockers, not for knowing the static on-disk fault layout.

3. Add owner-local coverage.
   - Add focused tests in `tests/ha/support/faults/mod.rs` for marker path mapping and marker create/remove behavior, including the “missing file is okay on remove” case.
   - Do not leave these assertions in `world/mod.rs`; the world owner should not test static fault-layout behavior.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4634 -6995 diff: -2361` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Tick the task boxes only after the gates pass.

### Guardrails

- Do not add a new wrapper struct for marker operations; plain faults-owned functions are enough.
- If the move starts pulling docker or `HarnessShared` concerns into `tests/ha/support/faults/mod.rs`, switch this plan back to `TO BE VERIFIED`.
- Keep `world/mod.rs` focused on scenario orchestration and runtime coordination; static fault-path layout should not remain there.

NOW EXECUTE
