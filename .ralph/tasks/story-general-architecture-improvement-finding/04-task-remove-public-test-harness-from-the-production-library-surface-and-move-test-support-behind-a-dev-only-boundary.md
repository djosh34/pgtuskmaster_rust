## Task: Remove Public `test_harness` From The Production Library Surface And Move Test Support Behind A Dev-Only Boundary <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
**Goal:** Remove `test_harness` from the public production library surface and move test support behind a boundary that is private to tests/dev support. The higher-order goal is to stop exporting large internal test-only helpers, process controls, real-binary provenance checks, and API/router assembly utilities as though they were part of the crate's supported runtime API. This task should make the production surface smaller and more intentional while keeping tests fully supported.

**Original general architectural request that this task must preserve:**
- "just like the dcs refactor task, i want a fully general improvement finding task"
- "make packages/mods more private"
- "reduce code interface between other components, make as small as possible interface"
- "find/checks/refactors radically internally to reduce code duplication. tries to simplify logic, de-spagthify, clean up old legacy logic/tests/shit"
- "untangle spagethi dependencies: just like dcs was controlled in many parts of the code, instead of a single worker. Find some other component that can be untangled, made almost fully private except very scoped/small interface, and thereby massively improving code quality, testability, reducing code in general (less code = better), cleaning up shit, making it more readable"

**Problem statement from current research:**
- `src/lib.rs` currently exports `#[doc(hidden)] pub mod test_harness;`. That means the crate is still publishing a large internal testing subsystem as part of its real library surface, only hidden from generated docs. Hidden docs are not privacy.
- The exported `src/test_harness/` tree contains broad capabilities that are clearly internal support code, including:
  - API router assembly helpers in `src/test_harness/api.rs`
  - real-binary provenance and policy verification in `src/test_harness/provenance.rs`
  - runtime config builders/samples in `src/test_harness/runtime_config.rs`
  - real etcd cluster spawning and lifecycle control in `src/test_harness/etcd3.rs`
  - TLS fixture generation in `src/test_harness/tls.rs`
- Integration tests currently depend on that public export. For example, `tests/bdd_api_http.rs` imports `pgtuskmaster_rust::test_harness::api::{build_test_router, build_test_router_with_live_state}` and `pgtuskmaster_rust::test_harness::runtime_config::RuntimeConfigBuilder`.
- This means the production crate surface is shaped by test convenience rather than by the smallest intentional runtime/library interface. In a greenfield repo with no compatibility burden, that is exactly the kind of leaked surface that should be eliminated rather than preserved.
- The current setup also makes it easier for internal production modules to lean on test-harness helpers instead of keeping a clean boundary between real code and dev-only support code.

**Concrete repo evidence from research:**
- `src/lib.rs`
  - `#[doc(hidden)] pub mod test_harness;`
- `src/test_harness/mod.rs`
  - re-exports many internal test-support modules (`api`, `auth`, `binaries`, `etcd3`, `namespace`, `pg16`, `ports`, `provenance`, `runtime_config`, `signals`, `tls`)
- `src/test_harness/api.rs`
  - builds routers by constructing `ApiServerCtx` and fake live state directly for tests
- `src/test_harness/runtime_config.rs`
  - contains a large public `RuntimeConfigBuilder` and many public sample helpers
- `src/test_harness/provenance.rs`
  - exposes `VerifiedRealBinaries` and real-binary verification helpers publicly through the library tree
- `src/test_harness/etcd3.rs`
  - exposes `EtcdClusterHandle` and cluster spawning helpers publicly through the library tree
- `tests/bdd_api_http.rs`
  - imports `test_harness` through the crate's public path today
- Other tests and internal test modules across `src/` also rely on `RuntimeConfigBuilder` and other harness helpers

**Required architectural direction:**
- The production library surface should stop exposing `test_harness` as a public module.
- Tests still need support code, but that support should live behind a dev-only/private boundary instead of a production-facing export.
- It is acceptable to solve this by moving test support into a dedicated test-support crate, dev-only support module, or another architecture that keeps tests working without keeping `test_harness` public in the main runtime crate.
- The result should favor the smallest deliberate runtime/library surface, not "public but undocumented" internals.

**Important non-goals for this task:**
- Do not break or downgrade existing tests just to hide the module.
- Do not replace strong real-binary or harness coverage with weaker fake/unit-only coverage.
- Do not stop at adding more `#[doc(hidden)]` or comments. The goal is a real privacy boundary in code structure.

**Scope:**
- Remove public `test_harness` exposure from `src/lib.rs`.
- Re-home or re-boundary the helpers currently living in `src/test_harness/` so tests can still use them without the main crate exporting them as production API.
- Update integration tests and any internal tests that currently import test-harness helpers through the public library path.
- Re-evaluate visibility inside the relocated/rewired test-support code so only the minimum necessary dev/test surface remains exposed.

**Expected outcome:**
- The main crate no longer exposes `test_harness` as part of its production library surface.
- Tests still have access to the support code they need, but through a boundary that is clearly dev/test-only.
- The public crate API is smaller, more intentional, and more private.
- Internal test support is easier to evolve without pretending it is runtime/library API.

</description>

<acceptance_criteria>
- [x] Remove `#[doc(hidden)] pub mod test_harness;` from `src/lib.rs` and replace it with a boundary that does not expose test support as production API.
- [x] Rework `src/test_harness/` ownership and visibility so helpers such as `api`, `runtime_config`, `etcd3`, `provenance`, `tls`, and related support code are no longer exported through the main library surface.
- [x] Update integration tests that currently import `pgtuskmaster_rust::test_harness::*`, including `tests/bdd_api_http.rs`, to use the new dev/test-only support boundary.
- [x] Update internal unit tests and support code across `src/` that currently depend on `RuntimeConfigBuilder` or other test-harness exports so they continue to work without relying on a public production export.
- [x] Preserve real-binary and harness-backed testing capabilities; do not weaken coverage just to simplify visibility.
- [x] Re-evaluate and reduce helper visibility inside the resulting test-support boundary so only the minimum necessary surface remains exposed to tests.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<plan>
- [x] Rename the leaked `src/test_harness` production export into an internal `src/dev_support` boundary, compile it only for unit tests or the dedicated dev-support feature, and stop exporting test support from the default runtime library surface.
- [x] Introduce a dedicated `pgtuskmaster_test_support` dev-only crate for integration-test consumption, re-point current integration tests at that crate, and keep internal unit tests on the crate-private `dev_support` boundary.
- [x] After the boundary/type rewrite is in place, repair compile fallout, reduce helper visibility inside the resulting support boundary, run `make check`, `make lint`, `make test`, `make test-long`, and only then update docs with the `k2-docs-loop` skill.
- [x] Docs review completed after validation: no published docs referenced `test_harness` or the production library test-support surface, so no docs page change was required for this internal boundary refactor.
</plan>
