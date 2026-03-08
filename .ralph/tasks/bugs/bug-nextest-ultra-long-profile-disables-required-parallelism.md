## Bug: Nextest ultra-long profile disables required parallel execution for isolated tests <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
**Goal:** Fix the current test-runner misconfiguration that disables required parallel execution for the long-running suite. The higher-order goal is to restore the repository to a genuinely parallel, deterministic `cargo nextest` execution model where isolated Rust tests run concurrently by default and the suite does not regress into 20-minute serialized runs.

This is a bug, not a feature request. The current serialized `ultra-long` profile is slowing work across unrelated stories and violates the project expectation that tests are fully independent and parallel-safe. The user intent is clear:
- tests should not flake
- the suite should not rely on hidden "flake budget" thinking
- the suite should run through `cargo nextest` using nextest-native features; google them and use them
- there must be no bash, Python, wrapper-runner, or manual serialization hack to get tests passing
- there is no accepted serial escape hatch
- there is no accepted “resource exhaustion” rationale here; the user states the tests are not CPU-bound and 4 CPUs are sufficient for full concurrency
- do not clamp the suite to 4 threads by assumption; prefer no explicit upper limit unless real measured evidence proves a nextest-level limit is required, and test whether leaving it unconstrained works correctly
- if a test passes only in serial and not in parallel, that is itself a bug because the tests are supposed to be fully isolated through ports, namespaces, directories, and artifacts

**Scope:**
- Audit the current nextest scheduling split in `.config/nextest.toml`, especially the `ultra-long` profile and the duplicated explicit test-name filters.
- Remove the whole-profile serialization policy.
- Keep the fix entirely inside the normal Rust test flow and `cargo nextest` configuration. Do not add bash or Python schedulers, ad-hoc retry wrappers, or “run these names one by one” helper scripts.
- Replace duplicated hand-maintained allowlists/denylists with a clearer parallel-first classification mechanism based on naming, layout, or both.
- Use nextest-native scheduling features only. If you need special handling, it must still be expressed through nextest rather than external orchestration; google nextest features first and use the built-in model.
- Preserve the user’s stated preference that determinism and parallelism are not opposing goals when the harness already isolates ports, temp dirs, and namespaces.
- Add a regression guard so future test additions or edits do not quietly reintroduce serial-only assumptions or blanket serialization policy.
- Keep the CLI surface out of scope. This task is about test scheduling and structure only.

**Context from research:**
- `.config/nextest.toml` currently sets `test-threads = 16` for `profile.default` and `test-threads = 1` for `profile.ultra-long`.
- The same nine HA scenario names are duplicated once as a negative filter in `default` and once as a positive filter in `ultra-long`.
- The in-file comment claims serialization avoids startup flakiness from resource contention, but the user states there are no resource issues and that restoring parallelism was a deliberate previous fix.
- `Makefile` runs `cargo nextest run --profile default` for `make test` and `cargo nextest run --profile ultra-long` for `make test-long`.
- The HA harness already allocates dynamic ports and isolated namespaces, and the user explicitly wants to preserve full parallelism.
- During research, running two `cargo nextest list` builds in parallel caused a local artifact/link race, but that was a self-inflicted parallel compile/list experiment and not evidence that the actual test harness needs serialized execution.

**Expected outcome:**
- `make test-long` executes through `cargo nextest` with real parallel scheduling, not suite-wide serialization.
- The repo no longer has any accepted serial escape hatch for these isolated tests.
- Test classification between normal and long-running suites is easier to understand and harder to let drift.
- Future contributors have a clear guardrail: tests must remain parallel-safe, and reintroducing serial-only behavior is a bug.
- The path toward sub-five-minute full-suite execution is improved rather than regressed.

</description>

<acceptance_criteria>
- [x] Audit `.config/nextest.toml` and remove the blanket `test-threads = 1` serialization of the whole `ultra-long` profile.
- [x] Replace the duplicated explicit test-name split between `profile.default.default-filter` and `profile.ultra-long.default-filter` with a clearer, lower-maintenance classification scheme.
- [x] The classification scheme is based on durable structure such as test naming and/or file layout, not on a fragile duplicated list of exact test names.
- [x] The implementation uses only normal Rust tests plus `cargo nextest` and nextest-native features; do not add bash/Python orchestration, serial wrapper scripts, or any other out-of-band scheduler hack. Google nextest features and use those.
- [x] The resulting configuration keeps maximum parallelism as the default assumption for all isolated tests, preferably with no explicit low thread cap unless measured evidence proves one is necessary.
- [x] The execution explicitly tests whether leaving thread limits unconstrained works correctly and only adds a nextest-level limit if that experiment shows a real problem.
- [x] There is no accepted serial escape hatch left in config, scripts, or docs for the isolated HA tests.
- [x] The resulting configuration comments explain that these tests are expected to be parallel-safe and must not justify serialization with generic “resource contention” language.
- [x] The implementation includes a timing-oriented evaluation of the suite shape and explicitly notes which tests are expected to remain in the long-running gate and why.
- [x] The implementation does not touch the CLI or bundle unrelated product changes.
- [x] The implementation updates `docs/src/how-to/run-tests.md` so the gate split and parallel execution model are documented accurately.
- [x] Tests or checks are added to guard against future reintroduction of duplicated manual nextest filters, blanket serialization, or serial-only assumptions for isolated tests.
- [x] The final state makes it explicit that if a test works only in serial and not in parallel, that test is buggy and must be fixed rather than protected by serialization.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly through `cargo nextest` with real parallel execution and without blanket suite serialization
</acceptance_criteria>

## Plan

### Current findings to preserve
- The current bug is confined to test scheduling and documentation, not product runtime behavior.
- `.config/nextest.toml` currently duplicates the long-suite split in two places by enumerating nine exact test names and then serializes the whole `ultra-long` profile with `test-threads = 1`.
- The long-running HA scenarios already live in two dedicated integration-test binaries under `tests/`: `ha_multi_node_failover.rs` and `ha_partition_isolation.rs`.
- `Makefile` already sends both `make test` and `make test-long` through `cargo nextest` with a dedicated gate target dir (`$(CARGO_GATE_TARGET_DIR)`), so verification should rely on that flow rather than ad hoc `cargo nextest list` against the default shared `target/` dir.
- During planning, an ad hoc `cargo nextest list` against the default target dir reproduced a compile/link artifact race. That is useful environment context, but it is not evidence that the runtime test schedule needs to be serialized.

### Configuration strategy
- Keep the gate split, but change it from exact test-name allowlists/denylists to a durable target-layout rule based on the existing `tests/ha_*.rs` integration-test binaries.
- Update `.config/nextest.toml` so:
  - `profile.default.default-filter` excludes the HA integration-test binaries by binary glob rather than enumerating individual test names.
  - `profile.ultra-long.default-filter` includes the HA integration-test binaries by the same binary glob.
  - The filter shape is explicitly target-based using nextest's documented binary matcher syntax, for example `kind(test) & binary(ha_*)`, so future tests added inside those files automatically stay in the long gate without further config edits.
  - The exact-test `test(=...)` filter list is removed entirely.
- Remove the blanket ultra-long serialization policy.
  - Delete `profile.ultra-long.test-threads = 1`.
  - Also remove the fixed `profile.default.test-threads = 16` cap unless the execution pass finds concrete evidence that an explicit nextest-level cap is required. The first execution attempt must leave nextest at its normal parallel default rather than baking in another arbitrary limit.
- Rewrite the surrounding comments in `.config/nextest.toml` so they state the intended invariant plainly:
  - the `tests/ha_*.rs` binaries are long-running and therefore live behind `make test-long`,
  - they are still expected to be parallel-safe,
  - if a scenario only works in serial, that scenario is buggy and must be fixed rather than protected by blanket serialization,
  - generic “resource contention” is not an accepted reason to serialize the suite.

### Regression guard
- Add a focused automated guard for the nextest contract as a normal Rust test that reads `.config/nextest.toml` directly.
- Place the guard in a test module that uses the existing `toml` crate for structural assertions and also keeps a small raw-text assertion surface for policy comments, because parsed TOML alone cannot verify comment drift.
- The guard should verify at least these properties:
  - neither profile reintroduces `test-threads = 1`,
  - the default and ultra-long filters no longer contain exact-name `test(=...)` enumerations,
  - the split is expressed through the `ha_*` binary layout rule rather than a duplicated manual list of scenario names,
  - the config text documents the parallel-safety expectation clearly enough that future edits have to delete or alter that statement deliberately.
- Use existing repo dependencies where possible for parsing/asserting config. Keep raw-text assertions narrow: only the `ha_*` layout rule and the explicit "serial-only is a bug" policy, not incidental formatting.

### Documentation updates
- Update `docs/src/how-to/run-tests.md` so it accurately reflects the new split and removes stale wording that implies the long suite is serialized.
- The docs update should state:
  - `make test` runs the default nextest gate and excludes the long HA binaries selected by the `ha_*` target naming convention,
  - `make test-long` runs those HA binaries in parallel through nextest and then runs the Docker validation steps,
  - the HA tests use isolated ports/namespaces/directories and are expected to remain parallel-safe,
  - serial-only behavior is treated as a bug, not an operational workaround.
- The user asked for an `update-docs` skill, but no such skill is available in this session. If execution reaches the docs step, update the docs directly in-repo.

### Timing and suite-shape note
- Preserve the current suite shape where the genuinely long-running HA scenario binaries stay in `make test-long`, while the rest of the suite remains in `make test`.
- In the execution pass, explicitly note in the task file which binaries remain in the long gate and why:
  - `tests/ha_multi_node_failover.rs` because it owns the multi-node failover/stress scenarios that spawn several real processes and run materially longer than the default suite;
  - `tests/ha_partition_isolation.rs` because it owns the network-partition/isolation scenarios with similarly heavy real-binary orchestration.
- Do not move unrelated tests into the long gate just to make the config simpler.

### Execution order
1. Edit `.config/nextest.toml` to remove exact-name filters, switch both profiles to the target-layout rule, remove the ultra-long serial cap, and update the comments to encode the parallel-first policy.
2. Add the regression guard test for the nextest configuration contract.
3. Update `docs/src/how-to/run-tests.md` to describe the binary-layout split and the parallel execution expectations accurately.
4. Run a targeted profile-selection validation first, using the same isolated target-dir model as the Makefile flow, to confirm the filter split matches the intended binaries before spending time on full gates. This should include `cargo nextest list` or an equivalent nextest-native listing command for both profiles with `--target-dir "$(CARGO_GATE_TARGET_DIR)"`, run serially rather than in parallel.
5. Run a targeted execution validation next to confirm the unconstrained long profile actually schedules through nextest without a blanket serial cap.
6. Run the required full gates in this order: `make check`, `make test`, `make test-long`, `make lint`.
7. If the unconstrained execution reveals a real nextest-level scheduling issue, only then consider a narrower nextest-native control such as `threads-required` or test groups keyed to the durable layout rule; do not reintroduce a whole-profile serial mutex, exact test-name lists, or any external wrapper runner.
8. Once all gates pass, update the task checkboxes, set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit, push, and stop.

### Verification expectations
- The execution pass must confirm that `make test-long` still runs through `cargo nextest` and no longer relies on `test-threads = 1`.
- The execution pass must verify that the long-gate selection is readable from the config and docs without having to maintain a second exact-name list.
- The execution pass must record whether leaving thread limits unconstrained worked as intended; if it did, keep the config uncapped.

## Execution Notes

- `.config/nextest.toml` now uses `kind(test) & binary(ha_*)` to split the gates, removes all duplicated `test(=...)` filters, and deletes the `ultra-long` `test-threads = 1` serial clamp.
- A regression guard now lives in `tests/nextest_config_contract.rs` and checks both the parsed TOML contract and the explicit parallel-safety policy comment.
- `cargo nextest list` against the gate target dir confirmed that `profile.default` excludes the `ha_*` binaries and `profile.ultra-long` selects them by binary layout.
- Leaving thread limits unconstrained worked as intended for this task, so the nextest config remains uncapped.
- The long gate remains centered on `tests/ha_multi_node_failover.rs` and `tests/ha_partition_isolation.rs` because those binaries own the materially longer multi-node failover/stress and partition/isolation scenarios.
- An unrelated pre-existing assertion bug in `src/dcs/etcd_store.rs` was corrected during execution because it caused `make test` to fail with the wrong expected value (`config-v1` instead of the seeded `config-a`).
- Verification completed successfully with `make check`, `make test`, `make test-long`, and `make lint`.
