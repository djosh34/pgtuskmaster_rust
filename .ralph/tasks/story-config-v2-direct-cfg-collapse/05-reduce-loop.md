## Task: Reduce Code Loop <status>completed</status> <passes>true</passes>

<description>
Your only goal is to reduce code and clean up the codebase.

use just-reduce-code skill
use improve-code-boundaries skill

This execution pass is not a generic cleanup. It is the next concrete reduction in the config-v2 direct-cfg collapse story:
- flatten the validated operator config root
- delete the wrapper corridor around `OperatorConfigV2`
- make the CLI consume that validated root directly instead of rebuilding endpoint/auth views downstream

End goal across this loop remains: remove 50-75% of code from `src/` and `tests/`.
This pass should still bias toward deletion, flattening, and reuse over moving code around.

Check current code reduction since commit `.ralph/git_diff_lines.txt` via:
`bash .ralph/git_diff_lines_since.sh`

Start number of lines:

- `src/`: 28,744 lines across 94 git-tracked files
- `tests/`: 9,331 lines across 53 git-tracked files
- Total (`src/` + `tests/`): 38,075 lines across 147 git-tracked files
</description>

<acceptance_criteria>
- [x] `src/config_v2/types.rs` no longer defines `OperatorApiEndpoint`
- [x] `src/config_v2/types.rs` no longer defines `ApiClientTokens`
- [x] `OperatorConfigV2` is the single validated operator root with top-level fields instead of nested endpoint/auth wrapper structs
- [x] `src/config_v2/parser/load_operator_config.rs` materializes the flat root directly and removes helper code that only existed to build the deleted wrappers
- [x] `src/cli/config.rs` consumes the flat root directly and no longer re-normalizes config-provided endpoint/auth data just because it was wrapped
- [x] Focused operator loader and CLI tests prove `base_url`, `expected_transport`, `resolve_to`, TLS merge behavior, and token behavior still work
- [x] `make check`
- [x] `make lint`
- [x] `make test`
- [x] `make test-long`
</acceptance_criteria>

<boundary_review>
Smell applied from `improve-code-boundaries`: wrong config-ingestion boundary.

Verified repository facts before execution:
- Task 01 explicitly required `OperatorConfigV2` to stay a flat validated operator root with top-level fields.
- Current `src/config_v2/types.rs` still exposes two wrapper shapes under that root:
  - `OperatorApiEndpoint { base_url, expected_transport, resolve_to }`
  - `ApiClientTokens { read_token, admin_token }`
- `OperatorApiEndpoint` is only consumed by `src/cli/config.rs`; it carries no independent behavior.
- `ApiClientTokens` is only used by `OperatorConfigV2` and `src/config_v2/parser/load_operator_config.rs`; it also carries no independent behavior.
- `src/config_v2/parser/load_operator_config.rs` already has to import a wide helper surface from `load_config.rs`, which is a sign the operator boundary is still split awkwardly.
- `src/cli/config.rs` currently has to compensate for the nested wrappers through `resolve_api_endpoint`, `resolve_config_auth`, `resolve_client_tls`, and transport/token normalization logic.
- That means validated config work is still leaking downstream: the operator loader is not handing the CLI one final operator root.

Verified type-reuse and reduction constraints:
- Do not add a new public operator DTO to replace the deleted wrappers.
- Reuse existing `OperatorConfigV2`, `OperatorClientTlsConfig`, `Secret`, and `PgtmApiTransportExpectation`.
- Prefer deleting wrapper structs over introducing replacement wrapper structs with different names.
- Keep CLI-only override behavior at the CLI boundary, but do not preserve extra config wrapper layers just to make override code feel symmetrical.

Verified design correction needed before execution:
- Flatten `OperatorConfigV2` so the validated root itself owns:
  - `base_url`
  - `expected_transport`
  - `resolve_to`
  - `client_tls`
  - `read_token`
  - `admin_token`
- Keep `OperatorClientTlsConfig` as the shared path-based TLS carrier.
- Delete `OperatorApiEndpoint` and `ApiClientTokens` instead of moving them.
- If execution proves that auth cannot stay as flat token fields without reintroducing complexity elsewhere, switch back to `TO BE VERIFIED` before inventing a new shared wrapper type.
</boundary_review>

<plan>
- [x] Change the type surface first in `src/config_v2/types.rs`:
  - remove `OperatorApiEndpoint`
  - remove `ApiClientTokens`
  - move their fields onto `OperatorConfigV2`
  - keep `api_auth_enabled()` only if it still reduces code after flattening
- [x] Update the config-v2 public surface and compile fallout next:
  - adjust `src/config_v2/mod.rs` exports
  - fix direct compile errors caused by the flattened operator root before changing unrelated behavior
- [x] Rewrite `src/config_v2/parser/load_operator_config.rs` to materialize the flat `OperatorConfigV2` directly:
  - map base URL / expected transport / resolve_to straight onto the root
  - map read/admin tokens straight onto the root
  - keep the existing merged TLS behavior
  - remove helper code that only existed to construct the deleted wrappers
- [x] Collapse `src/cli/config.rs` onto the flat operator root:
  - replace wrapper-specific resolution helpers with one direct override application path
  - keep transport validation only where a CLI `--base-url` override can still change the resolved scheme
  - stop cloning or re-normalizing config-provided token strings once the validated config already owns the final values
- [x] Update focused operator parser and CLI tests so they prove the flattened root still preserves:
  - `base_url`
  - `expected_transport`
  - `resolve_to`
  - merged TLS identity/CA behavior
  - read/admin token behavior for config-only and CLI-override cases
- [x] Run the required gates in order and tick them when they pass:
  - `make check`
  - `make lint`
  - `make test`
  - `make test-long`
- [ ] If execution shows that one deleted wrapper carried indispensable shared behavior, change the plan tail back to `TO BE VERIFIED`, document the exact missing shape here, and stop immediately

NOW EXECUTE
</plan>
