## Bug: Docker helper scripts ignore command failures during readiness and cleanup <status>not_started</status> <passes>false</passes>

<description>
The Docker helper flow currently contains ignored-error patterns that hide real failures instead of handling them explicitly.

Examples found during task planning:
- `tools/docker/common.sh` uses `curl ... || true` inside `wait_for_ha_member_count`, which can mask transport and HTTP failures while polling.
- `tools/docker/smoke-cluster.sh` cleanup uses `compose_down ... || true`, which suppresses teardown failures entirely.

This repository explicitly disallows swallowing or ignoring errors. Explore the Docker helper and smoke scripts first, then replace the ignored-error patterns with explicit, intentional handling that preserves useful diagnostics without making teardown noisy or flaky.
</description>

<acceptance_criteria>
- [ ] Ignored-error patterns in the Docker helper/smoke scripts are removed or replaced with explicit handling
- [ ] Cleanup and polling behavior still produce understandable output during success and failure paths
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
