---
## Bug: Provenance Helpers Missing Break Lib Test Compile <status>done</status> <passes>true</passes>

<description>
`src/test_harness/provenance.rs` calls helper functions that are not defined in scope:
- `verify_policy_optional_pins`
- `verify_attestation_metadata`
- `verify_attested_entry_metadata`

Detected while running:
`cargo test provenance_accepts_valid_attested_binary -- --exact`

Compiler errors:
- `E0425 cannot find function verify_policy_optional_pins`
- `E0425 cannot find function verify_attestation_metadata`
- `E0425 cannot find function verify_attested_entry_metadata`

Please explore and research the codebase first, then implement a fix that restores compilation and keeps provenance validation behavior coherent.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
