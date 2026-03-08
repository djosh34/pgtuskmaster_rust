# Verbose context for docs/src/tutorial/debug-api-usage.md

Core behavior:
- The debug API worker builds a full snapshot from current config, PostgreSQL state, DCS state, process state, and HA state.
- It records change events and timeline entries whenever the summarized signatures for those domains change.
- The worker keeps a monotonically increasing sequence counter for change/timeline entries.
- The default in-memory retention limit is `300` entries for `changes` and `300` entries for `timeline`.

Retention and history details from the code:
- `src/debug_api/worker.rs` defines `DEFAULT_HISTORY_LIMIT: usize = 300`.
- `trim_history()` pops from the front of both deques whenever their length exceeds `history_limit`.
- Each call to `record_change(...)` appends one `DebugChangeEvent` and one `DebugTimelineEntry` with the same sequence number.

How `since=` works:
- The controller layer maps debug state into API responses and exposes `/debug/verbose`.
- Existing repo docs already describe the intended contract as "include only `changes` and `timeline` entries where `sequence > since` while leaving the top-level snapshot at the latest state."
- The choose-doc request specifically asked to explain incremental polling; the observer test harness is the right mental model to explain it:
  - first request without `since` or with `since=0`
  - read `meta.sequence`
  - next request uses `?since=<previous sequence>`
  - only newer changes/timeline entries are returned
- The tutorial should teach this as snapshot-plus-incremental-history polling, not as a delta-only API.

Availability and auth:
- The cluster runtime sample at `docker/configs/cluster/node-a/runtime.toml` has `debug.enabled = true`.
- Debug endpoints are only meant to exist when debug mode is enabled.
- The API layer and the existing debug API reference indicate the debug endpoints share the main HTTP listener and therefore inherit the API TLS/auth posture rather than introducing a separate listener.
- In the provided cluster sample, the API listener is `0.0.0.0:8080`.
- The compose examples then publish host ports such as `18081` to that internal API listener.

Explicit rate limits or client-side thresholds:
- I did not find an explicit rate limiter, poll budget, or backpressure threshold in the requested files.
- The most concrete operational limit in the code is the in-memory history depth of 300 entries for each stream.
- Safe wording for docs: there is no explicit rate-limit contract in the requested code, so clients should poll conservatively and rely on `since` to avoid repeatedly transferring full history arrays.

What the observer test harness implies about intended use:
- `tests/ha/support/observer.rs` is used as a codebase example of polling system state during scenarios.
- That makes it a good narrative anchor for a tutorial: treat the debug API as a read-only observation surface for state transitions, failover timelines, and trust changes.
- The tutorial should show users how to inspect a stable snapshot first, then watch specific parts (`meta`, `dcs`, `ha`, `changes`, `timeline`) over time.

Suggested factual boundaries for the tutorial:
- Do not claim that every poll returns only deltas; the top-level state is still a current snapshot.
- Do not invent rate limits or server-side cache semantics that are not in the requested files.
- Do not imply a dedicated debug port; the code and docs point to the normal API listener.
- Do explain that the retention window is finite, so very old sequences can age out.

Useful fields to orient the reader:
- `meta`: current snapshot sequence and environment metadata.
- `dcs`: trust, member count, leader, switchover presence.
- `ha`: phase and decision shape.
- `changes`: structured "what changed" records with previous/current versions and summaries.
- `timeline`: chronological event feed derived from the same change detection.
