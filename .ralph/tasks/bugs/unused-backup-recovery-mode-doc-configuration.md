---
## Bug: backup.bootstrap.recovery_mode is documented but unused <status>blocked</status> <passes>false</passes>

<blocked_by>05-task-remove-backup-docs-and-obsolete-task-artifacts</blocked_by>

<description>
This bug is intentionally deferred until the backup-removal story is fully complete. The entire backup/restore config surface is scheduled for deletion, so this specific dead knob should be reassessed only after the story finishes end-to-end.

Reassess this bug after the final task in `story-remove-backup-feature`:
- if the field is gone everywhere, close this as obsolete,
- if any backup/restore vocabulary or docs survive unexpectedly, file a narrower cleanup bug against the remaining surface.

Current concern recorded here: `backup.bootstrap.recovery_mode` is documented in `docs/src/operator/configuration.md` as part of restore bootstrap config and accepted in schema/defaults, but runtime code does not read or branch on this field anywhere in startup/restore paths. It is effectively dead configuration that cannot affect behavior, which makes the docs and operator guidance inaccurate for current behavior.

What to explore:
- Trace all uses of `backup.bootstrap.recovery_mode` in `src/` to confirm it is not consumed.
- Confirm whether restore behavior should support multiple recovery modes or whether the field should be removed from schema/docs.

Potential fixes:
- If a mode is intended, implement behavior behind the field and document supported values.
- If not intended, remove the field from schema/defaults/docs and migration path, and update docs with a clear note that this knob is not supported.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
