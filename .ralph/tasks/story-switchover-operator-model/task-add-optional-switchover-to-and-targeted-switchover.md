## Task: Add Optional `switchover_to` And Targeted Switchover Support <status>done</status> <passes>true</passes>

<priority>low</priority>

<description>
Add an operator-meaningful switchover request shape with optional `switchover_to`, preserve generic switchover behavior when the target is omitted, and ensure the repository documents only the current semantics.

Current implementation direction:
- DCS stores `SwitchoverRequest { switchover_to: Option<MemberId> }`.
- API accepts `{}` for generic requests and `{"switchover_to":"member-id"}` for targeted requests.
- CLI supports `ha switchover request --switchover-to <member-id>`.
- HA state exposes `switchover_pending` and `switchover_to`.
- HA leadership attempts stay generic when no target is supplied, and during a targeted switchover only the requested eligible replica may acquire leadership.
</description>

<acceptance_criteria>
- [x] Production switchover paths use the optional `switchover_to` request model in DCS, API, CLI, and HA state.
- [x] Generic switchovers still work with `{}` and retain the previous automatic successor behavior.
- [x] Targeted switchovers validate unknown, empty, leader, and ineligible targets with clear API failures.
- [x] CLI exposes the optional target flag and renders `switchover_to` in HA state output.
- [x] HA decision tests cover targeted gating and preserve the generic observed-primary follow path.
- [x] Stale draft, prompt, and temporary documentation that described superseded switchover semantics has been removed or rewritten.
- [x] Repository-wide verification shows no tracked `requested_by` or `switchover_requested_by` references remain.
- [x] `make check` passes.
- [x] `make test` passes.
- [x] `make test-long` passes.
- [x] `make lint` passes.
</acceptance_criteria>

<plan>
Execution summary:
1. Replace the marker request with `switchover_to: Option<MemberId>` and thread it through DCS encode/decode paths.
2. Use a shared eligible-target helper for API validation and targeted HA leadership gating.
3. Accept both generic and targeted HTTP/CLI requests while preserving strict request parsing.
4. Expose `switchover_pending` plus `switchover_to` in HA state and CLI text/json output.
5. Preserve generic follow behavior for observed primaries and add targeted switchover candidate gating.
6. Update tests, live docs, and tracked task/doc artifacts to describe only the current model.
7. Run the required verification gates, then mark the task complete and switch tasks.
</plan>

NOW EXECUTE
