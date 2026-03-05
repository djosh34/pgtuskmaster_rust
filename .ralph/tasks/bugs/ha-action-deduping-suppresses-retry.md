---
## Bug: HA action dedupe suppresses legitimate retries <status>not_started</status> <passes>false</passes>

<description>
`HaState.recent_action_ids` is only ever appended to and never cleared in normal operation. `decide` filters candidate actions by this set, so once an action was emitted once (for example `StartPostgres`, `StartRewind`, `RunBootstrap`, or a future restore action), the same action can never be retried later if it failed or if the state machine returns to that decision again. This can stall recovery loops because repeated process-triggering actions are silently dropped as duplicates.</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
