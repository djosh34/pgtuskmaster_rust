## Task: [Improvement] Type network endpoints instead of carrying raw strings across runtime <status>not_started</status> <passes>false</passes>

<description>
The codebase carries API and DCS endpoint addresses as raw `String` values deep into runtime and harness paths, then parses or binds them at scattered call sites. This was detected during a representation-integrity scan looking for cases where subsystem boundaries retain ad-hoc primitive encodings instead of canonical typed models.

Examples of the drift surface:
- `src/config/schema.rs` stores `api.listen_addr: String` and `dcs.endpoints: Vec<String>`.
- `src/runtime/node.rs` binds `cfg.api.listen_addr.as_str()` directly at worker startup.
- `src/dcs/etcd_store.rs` accepts `Vec<String>` endpoints and clones/reuses them throughout the worker/store path.
- `src/test_harness/ha_e2e/util.rs` reparses endpoint strings with `parse_http_endpoint`, showing that the same concept remains untyped until late.

Explore the current endpoint/address flow first, decide the smallest coherent typed model or models for these network addresses, then refactor the internal paths to use typed representations and keep string encoding only at true external boundaries if still required by third-party APIs.
</description>

<acceptance_criteria>
- [ ] Endpoint/address flows are mapped across config, runtime, DCS store, API startup, and harness code before edits begin
- [ ] Internal API listen-address handling no longer depends on raw `String` values where a typed socket/address model is more appropriate
- [ ] Internal DCS endpoint handling no longer carries unvalidated raw endpoint strings across runtime/store paths where a typed endpoint model is more appropriate
- [ ] Any remaining string encoding is limited to explicit external boundaries and is justified by the target library/API contract
- [ ] Tests are updated or added to cover the typed endpoint flow and any normalization/validation behavior introduced
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
