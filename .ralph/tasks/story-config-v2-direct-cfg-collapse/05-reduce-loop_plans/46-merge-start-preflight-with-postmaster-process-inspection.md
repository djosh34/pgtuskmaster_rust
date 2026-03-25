## Plan: Merge Start Preflight With Postmaster Process Inspection

### Why this reduction target

`src/process/worker.rs` still owns postgres-process inspection even though `src/process/postmaster.rs` already owns the authoritative managed-postmaster verification boundary.

- `start_postgres_preflight_is_already_running(...)` re-reads `postmaster.pid`, distinguishes stale pidfiles, and walks `/proc/.../cmdline` to decide whether a postgres process is already present.
- `lookup_managed_postmaster(...)` already parses `postmaster.pid`, verifies the process is running, validates that it is really postgres/postmaster, and confirms it matches the managed data directory.
- The worker then adds a second socket-lock pid parser and a second `/proc` command-line check just to decide whether a stale socket lock belongs to postgres.

That is boundary duplication, not distinct behavior. The ownership for "is this pid a live postgres process and does it belong to this managed node?" should live in one place, and the worker should only orchestrate the answer.

### Current overlap already verified

- `src/process/worker.rs` duplicates `/proc/{pid}/cmdline` inspection in `pid_is_postgres_process(...)`.
- `src/process/worker.rs` parses a socket lock file in `parse_postgres_socket_lock_pid(...)` and then re-validates the pid with its own postgres-process heuristic.
- `src/process/postmaster.rs` already owns pid parsing, pid liveness checks, and postgres argv/data-dir validation in `lookup_managed_postmaster(...)`, `parse_postmaster_pid(...)`, `pid_matches_data_dir(...)`, and `pid_exists(...)`.
- The current worker preflight treats stale pid files and stale socket locks as cleanup concerns, but the actual process-identification logic is split across two files with different error vocabularies.

### Execution plan

1. Move postgres pid inspection into `src/process/postmaster.rs`.
   - Reuse `ManagedPostmasterPid` and the existing `/proc` inspection path instead of keeping `worker.rs`'s separate `pid_is_postgres_process(...)`.
   - Add one small postmaster-owned helper that answers whether an arbitrary pid currently belongs to a postgres/postmaster process.
   - Keep the existing `ManagedPostmasterError` boundary authoritative for process-inspection failures; do not invent a parallel pid-probing enum in the worker.

2. Move start-preflight probing next to postmaster verification.
   - Introduce a postmaster-owned preflight helper that:
     - checks the managed `postmaster.pid` via `lookup_managed_postmaster(...)`,
     - cleans stale `postmaster.pid` / `postmaster.opts` when lookup proves the pidfile is stale,
     - parses the socket lock pid,
     - reuses the new postmaster helper to decide whether that lock still belongs to postgres,
     - removes stale socket and lock files only when no live postgres process remains.
   - Keep the worker-facing API focused on the preflight result, not on filesystem details.

3. Shrink `src/process/worker.rs` back to orchestration.
   - Delete `pid_is_postgres_process(...)`, `parse_postgres_socket_lock_pid(...)`, `cleanup_postgres_socket_files(...)`, `start_postgres_preflight_is_already_running(...)`, and the worker-local file-removal wrappers if the new postmaster API subsumes them.
   - Replace `start_postgres_preflight_details(...)` plus the current match block with a thin worker call that translates the reused preflight result into:
     - no-op success when postgres is already running,
     - failure outcome when postmaster-owned preflight returns an error,
     - normal launch when cleanup/preflight reports "safe to start".

4. Reuse existing fake-postgres coverage instead of growing test scaffolding.
   - Move or rewrite the current preflight tests so they exercise the new postmaster-owned helper and still cover:
     - existing managed postgres leading to start no-op,
     - stale pidfile cleanup,
     - stale socket lock cleanup,
     - live socket-lock postgres preventing cleanup.
   - Keep `worker.rs` tests focused on worker outcomes and let `postmaster.rs` own process-probing edge cases.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not create a new public process-inspection module if `postmaster.rs` can own the shared logic privately.
- Do not keep two different `/proc` argv parsers after this slice; worker and postmaster must share one owner.
- Do not add a new DTO just to report "already running vs safe to start" if a small existing-style enum in `postmaster.rs` is sufficient.
- If the preflight helper starts mixing spawn/orchestration concerns back into `postmaster.rs`, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
