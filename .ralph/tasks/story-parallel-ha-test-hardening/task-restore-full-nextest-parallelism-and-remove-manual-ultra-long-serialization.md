## Task: Restore Full Nextest Parallelism And Remove Manual Ultra-Long Serialization <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Restore the test suite to a genuinely parallel execution model and remove the current blanket serialization of the ultra-long profile. The higher-order goal is to make the full automated suite fast, deterministic, and intentionally parallel by design, with a target direction of all tests completing within roughly five minutes wall-clock on a healthy development machine or CI worker, rather than treating slow or serialized execution as acceptable policy.

This task is not about trading correctness for speed. It is about removing an apparent regression in test scheduling policy and replacing it with an explicit, maintainable, parallel-first structure. The user intent is clear:
- tests should not flake
- the suite should not rely on hidden "flake budget" thinking
- the suite should remain fully parallel where isolation already exists through per-test ports, namespaces, and directories
- `test-threads = 1` for the whole ultra-long profile should be treated as a bug or an overcorrection unless concrete resource coupling is proven

**Scope:**
- Audit the current nextest scheduling split in `.config/nextest.toml`, especially the `ultra-long` profile and the duplicated explicit test-name filters.
- Remove the whole-profile serialization policy unless there is a narrowly scoped, demonstrated reason to keep a specific test or test family constrained.
- Replace duplicated hand-maintained allowlists/denylists with a clearer parallel-first classification mechanism based on naming, layout, or both.
- Prefer nextest-native scheduling controls for exceptional cases only. If any tests truly need special handling, encode that with targeted per-test policy rather than by serializing the entire long-running suite.
- Preserve the user’s stated preference that determinism and parallelism are not opposing goals when the harness already isolates ports, temp dirs, and namespaces.
- Keep the CLI surface out of scope. This task is about test scheduling and structure only.

**Context from research:**
- `.config/nextest.toml` currently sets `test-threads = 16` for `profile.default` and `test-threads = 1` for `profile.ultra-long`.
- The same nine HA scenario names are duplicated once as a negative filter in `default` and once as a positive filter in `ultra-long`.
- The in-file comment claims serialization avoids startup flakiness from resource contention, but the user states there are no resource issues and that restoring parallelism was a deliberate previous fix.
- `Makefile` runs `cargo nextest run --profile default` for `make test` and `cargo nextest run --profile ultra-long` for `make test-long`.
- The HA harness already allocates dynamic ports and isolated namespaces, and the user explicitly wants to preserve full parallelism.
- During research, running two `cargo nextest list` builds in parallel caused a local artifact/link race, but that was a self-inflicted parallel compile/list experiment and not evidence that the actual test harness needs serialized execution.

**Expected outcome:**
- The repo no longer serializes the entire `ultra-long` profile.
- Test classification between normal and long-running suites is easier to understand and harder to let drift.
- Any remaining scheduling exceptions are narrow, justified, and encoded in a way that does not throw away global parallelism.
- The path toward sub-five-minute full-suite execution is improved rather than regressed.

</description>

<acceptance_criteria>
- [ ] Audit `.config/nextest.toml` and remove the blanket `test-threads = 1` serialization of the whole `ultra-long` profile unless a specific test-level exception is proven necessary and documented.
- [ ] Replace the duplicated explicit test-name split between `profile.default.default-filter` and `profile.ultra-long.default-filter` with a clearer, lower-maintenance classification scheme.
- [ ] The classification scheme is based on durable structure such as test naming and/or file layout, not on a fragile duplicated list of exact test names.
- [ ] If nextest per-test scheduling controls are needed, they are applied only to the specific exceptional tests rather than to the whole long-running suite.
- [ ] The resulting configuration keeps maximum parallelism as the default assumption for all isolated tests.
- [ ] The resulting configuration documents, in comments, the exact reason for any remaining constrained tests, and those comments must not hand-wave with generic “resource contention” language.
- [ ] The implementation includes a timing-oriented evaluation of the suite shape and explicitly notes which tests are expected to remain in the long-running gate and why.
- [ ] The implementation does not touch the CLI or bundle unrelated product changes.
- [ ] The implementation updates `docs/src/how-to/run-tests.md` so the gate split and parallel execution model are documented accurately.
- [ ] Tests or checks are added, if practical, to guard against future reintroduction of duplicated manual nextest filters or blanket serialization.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly and runs without blanket suite serialization
</acceptance_criteria>
