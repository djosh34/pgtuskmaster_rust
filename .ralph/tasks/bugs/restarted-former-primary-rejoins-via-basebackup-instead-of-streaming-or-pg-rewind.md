## Bug: Restarted former primary rejoins via basebackup instead of streaming or pg_rewind <status>not_started</status> <passes>false</passes>

<description>
During a live manual HA exercise against `tests/ha/givens/three_node_plain/compose.yml`, the old primary was killed, a new primary was elected, and then the old primary container was restarted with its volume still present. The restarted node rejoined as a replica, but `pgtm status` showed it going through `follower(Basebackup)` before eventually reaching `follower(StartStreaming)`.

That behavior is suspicious for a normal former-primary rejoin with an existing data directory. The operator expectation is that the node should first try the cheaper and more topology-faithful recovery paths:
- resume as a replica directly when the local data directory is still a consistent replica candidate
- otherwise attempt `pg_rewind` when divergence is rewindable
- fall back to `pg_basebackup` only when streaming/rewind are impossible or have already failed

In the observed run there was no explicit evidence of a streaming-first or rewind-first attempt before the node entered the basebackup path. The relevant decision logic appears to live in:
- `src/ha/decide.rs`, where `follow_goal` selects `RecoveryPlan::Basebackup`, `RecoveryPlan::StartStreaming`, or `RecoveryPlan::Rewind`
- `src/ha/reconcile.rs`, where the chosen recovery plan becomes `BaseBackup`, `PgRewind`, or `StartReplica`

Please explore and research the codebase first, then fix. The fix should make the restarted former primary prefer direct replica start or `pg_rewind` when its on-disk state permits that, and only use `pg_basebackup` as the last resort. Add or tighten HA coverage so the normal old-primary rejoin path proves that `pg_basebackup` is not chosen unless the local state truly requires it, while the explicit `ha_rewind_fails_then_basebackup_rejoins_old_primary` scenario still covers the forced fallback path.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
