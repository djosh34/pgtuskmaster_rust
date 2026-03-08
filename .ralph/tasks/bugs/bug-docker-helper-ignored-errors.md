## Bug: Docker helper scripts ignore command failures during readiness and cleanup <status>completed</status> <passes>true</passes>

<description>
The Docker helper flow currently contains ignored-error patterns that hide real failures instead of handling them explicitly.

Examples found during task planning:
- `tools/docker/common.sh` uses `curl ... || true` inside `wait_for_ha_member_count`, which can mask transport and HTTP failures while polling.
- `tools/docker/smoke-cluster.sh` cleanup uses `compose_down ... || true`, which suppresses teardown failures entirely.

This repository explicitly disallows swallowing or ignoring errors. Explore the Docker helper and smoke scripts first, then replace the ignored-error patterns with explicit, intentional handling that preserves useful diagnostics without making teardown noisy or flaky.
</description>

<acceptance_criteria>
- [x] Ignored-error patterns in the Docker helper/smoke scripts are removed or replaced with explicit handling
- [x] Cleanup and polling behavior still produce understandable output during success and failure paths
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Execution Plan

### Current code facts to preserve
- `tools/docker/common.sh` already implements the real Docker Compose teardown in `compose_down()` and the HA readiness polling in `wait_for_ha_member_count()`, so the fix should stay centered there rather than duplicating shell logic in each caller.
- `tools/docker/cluster.sh` already uses the stricter cleanup/teardown style that this bug wants: it lets `compose_down()` fail normally for the persistent `down` command instead of suppressing errors.
- The actual ignored-error surface is broader than the task description examples:
  - `tools/docker/smoke-cluster.sh` cleanup currently logs a generic warning if `compose_down()` fails, but it does not include actionable diagnostics from Docker.
  - `tools/docker/smoke-single.sh` still contains the fully swallowed pattern `compose_down ... >/dev/null 2>&1 || true`.
- The current HA polling loop in `wait_for_ha_member_count()` retries both transport failures and semantic-not-ready responses in one branch. That keeps the smoke tests resilient, but it leaves the final timeout message too weak because it does not tell the operator whether the problem was HTTP reachability, malformed JSON, or a valid-but-not-ready HA state.

### Implementation steps
1. Refactor the shared Docker helper in `tools/docker/common.sh` so failure handling is explicit instead of implicit.
   - Add a small shared helper that runs `compose_down()`, captures its stderr/stdout, and returns both status and bounded diagnostic text to the caller instead of printing a generic warning.
   - Do not make trap semantics magical inside the shared helper. Each smoke script cleanup must preserve its incoming `$?`, call the shared helper inside an explicit `if ! ...; then` branch, print the helper-provided diagnostics, remove the temp root, and finally `return` the original status so cleanup reporting never overwrites the real failure.
   - Rework `wait_for_ha_member_count()` so it distinguishes among:
     - transport or HTTP failures from `curl`,
     - empty or malformed responses,
     - valid JSON responses whose HA fields have not converged yet.
   - Preserve polling behavior during the timeout window, but retain the most recent failure reason and include it in the final timeout message so the operator gets concrete diagnostics.

2. Update both smoke scripts to use the explicit helper behavior and remove all swallowed errors.
   - Change `tools/docker/smoke-single.sh` cleanup from `compose_down ... || true` to explicit exit-status-preserving cleanup that only attempts teardown when the env file exists, reports any failure clearly, and always removes the temp root.
   - Tighten `tools/docker/smoke-cluster.sh` cleanup so it uses the same explicit cleanup structure as the single-node script rather than an inline generic warning message.
   - Keep the smoke scripts ephemeral and trap-based; do not change their lifecycle contract into a persistent cluster flow.

3. Keep success-path output quiet, but make failure-path output more useful.
   - Do not dump verbose polling logs on every retry.
   - On final HA readiness timeout, print the last observed failure reason and, when available, the last response body or parsed HA summary that explains why readiness never completed.
   - On cleanup failure, print the Compose project name and the actual Docker error text instead of a generic warning.

4. Check whether any docs or help text mention behavior that is now stale, and update only the operator-facing pages that actually need it.
   - Review `README.md`, `docs/src/tutorial/first-ha-cluster.md`, and `docs/src/how-to/check-cluster-health.md` for any statements that imply smoke cleanup silently succeeds or that hide how failures surface.
   - No `update-docs` skill is available in this session even though the task checklist mentions it, so the execution pass should update docs directly in-repo if wording changes are needed.
   - Regenerate the built docs artifacts if source docs change so `docs/book/` does not drift from `docs/src/`.

5. Verify with the full required gates after implementation, not just targeted shell inspection.
   - Run targeted script checks first where useful, such as `bash -n` on the edited scripts and the relevant Docker smoke target or direct script invocation, to confirm the new error-reporting path behaves sensibly.
   - Then run the required full gates exactly as requested:
     - `make check`
     - `make test`
     - `make test-long`
     - `make lint`
   - Only after all of those pass should the execution turn mark acceptance boxes complete, set `<passes>true</passes>`, run `.ralph/task_switch.sh`, commit, and push.

### Constraints for the execution turn
- Do not re-explore the codebase broadly once this plan is approved; execute against `tools/docker/common.sh`, `tools/docker/smoke-single.sh`, `tools/docker/smoke-cluster.sh`, and only the minimal docs files that genuinely need updates.
- Do not introduce `|| true`, ignored stderr, `unwrap`, `expect`, `panic`, or equivalent swallowed-error patterns anywhere in the fix.
- Prefer shared helper refactoring over per-script special cases so the single-node and cluster smoke flows do not drift again.

NOW EXECUTE
