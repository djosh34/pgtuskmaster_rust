## Bug: cli_binary tests cannot spawn built executables <status>not_started</status> <passes>false</passes>

<description>
`make test` currently fails in `tests/cli_binary.rs` because multiple tests hit `No such file or directory (os error 2)` when spawning the expected CLI binaries after path resolution.

Observed failures:
- `missing_required_subcommand_arg_exits_usage_code`
- `node_help_exits_success`
- `node_rejects_empty_dcs_basic_auth_username_with_stable_field_path`
- `node_missing_secure_field_prints_stable_field_path`
- `node_rejects_https_dcs_without_tls_config`
- `status_command_uses_state_endpoint`
- `status_auth_failure_maps_to_exit_4`
- `switchover_clear_uses_delete_switchover_endpoint`

The failing harness lives in `tests/cli_binary.rs` and resolves binary paths via `CARGO_BIN_EXE_*` or a fallback derived from `current_exe()`. Explore and research the codebase first, then fix the executable discovery/spawn boundary so the suite can run reliably under `make test`.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
