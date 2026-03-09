## Bug: HA replica must not follow non-authoritative primary <status>not_started</status> <passes>false</passes>

<description>
In the HA decision and startup-rejoin paths, source selection falls back from the authoritative
leader lease to any healthy primary member record in DCS. This lets a stale former primary, or a
hard-killed node's still-fresh old member record, become a follow source for healthy replicas when
the real leader is briefly not considered active. Investigate the HA decision, process dispatch,
and runtime startup/rejoin codepaths first, then remove non-authoritative primary fallback so
replicas only follow the leased leader (or stay quiescent while no authoritative leader exists).
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
