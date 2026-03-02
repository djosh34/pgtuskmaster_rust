---
## Task: Implement DCS worker trust evaluation cache updates and member publishing <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Implement DCS ownership rules: trust evaluation, typed key parsing, cache updates, and local member publishing.

**Scope:**
- Implement `src/dcs/keys.rs`, `src/dcs/state.rs`, `src/dcs/store.rs`, `src/dcs/worker.rs`, `src/dcs/mod.rs`.
- Implement `evaluate_trust`, `build_local_member_record`, `apply_watch_update`, `key_from_path`, `write_local_member`, and `refresh_from_etcd_watch`.
- Ensure DCS worker subscribes directly to pginfo watch and owns `/member/{self_id}` writes.

**Context from research:**
- Plan explicitly forbids external `upsert_member(...)` APIs.

**Expected outcome:**
- DCS state is authoritative and consistent with typed watch and etcd events.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] `DcsTrust` and `DcsState` exactly reflect plan semantics.
- [ ] Key parsing rejects malformed paths with typed errors.
- [ ] Tests cover quorum transitions (`FullQuorum`, `FailSafe`, `NotTrusted`).
- [ ] Integration tests verify local member publish on pginfo version change.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] On failures, create `$add-bug` tasks for each distinct defect.
</acceptance_criteria>
