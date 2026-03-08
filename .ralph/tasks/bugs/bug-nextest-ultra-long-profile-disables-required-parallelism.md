## Bug: Nextest ultra-long profile disables required parallel execution for isolated tests <status>not_started</status> <passes>false</passes>

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
- [ ] Audit `.config/nextest.toml` and remove the blanket `test-threads = 1` serialization of the whole `ultra-long` profile.
- [ ] Replace the duplicated explicit test-name split between `profile.default.default-filter` and `profile.ultra-long.default-filter` with a clearer, lower-maintenance classification scheme.
- [ ] The classification scheme is based on durable structure such as test naming and/or file layout, not on a fragile duplicated list of exact test names.
- [ ] The implementation uses only normal Rust tests plus `cargo nextest` and nextest-native features; do not add bash/Python orchestration, serial wrapper scripts, or any other out-of-band scheduler hack. Google nextest features and use those.
- [ ] The resulting configuration keeps maximum parallelism as the default assumption for all isolated tests, preferably with no explicit low thread cap unless measured evidence proves one is necessary.
- [ ] The execution explicitly tests whether leaving thread limits unconstrained works correctly and only adds a nextest-level limit if that experiment shows a real problem.
- [ ] There is no accepted serial escape hatch left in config, scripts, or docs for the isolated HA tests.
- [ ] The resulting configuration comments explain that these tests are expected to be parallel-safe and must not justify serialization with generic “resource contention” language.
- [ ] The implementation includes a timing-oriented evaluation of the suite shape and explicitly notes which tests are expected to remain in the long-running gate and why.
- [ ] The implementation does not touch the CLI or bundle unrelated product changes.
- [ ] The implementation updates `docs/src/how-to/run-tests.md` so the gate split and parallel execution model are documented accurately.
- [ ] Tests or checks are added to guard against future reintroduction of duplicated manual nextest filters, blanket serialization, or serial-only assumptions for isolated tests.
- [ ] The final state makes it explicit that if a test works only in serial and not in parallel, that test is buggy and must be fixed rather than protected by serialization.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly through `cargo nextest` with real parallel execution and without blanket suite serialization
</acceptance_criteria>
