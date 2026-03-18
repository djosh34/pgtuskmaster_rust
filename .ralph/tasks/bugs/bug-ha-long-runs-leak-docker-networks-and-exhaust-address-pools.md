## Bug: HA long runs leak Docker networks and exhaust address pools <status>not_started</status> <passes>false</passes>

<description>
`make test-long` can fail before scenario execution because Docker cannot allocate another compose network:

- `Error response from daemon: all predefined address pools have been fully subnetted`
- on this host, `docker network ls` showed 28 leftover `ha-*` networks when the failure happened
- the failed scenarios were aborting in the initial `Given the "three_node_plain" harness is running` step while `docker compose` tried to create the per-scenario network

Explore and research the HA harness cleanup path, docker compose project/network lifecycle, and the failure/abort paths in long parallel runs first, then fix the leak so repeated `make test-long` runs do not accumulate orphaned `ha-*` networks or exhaust Docker address pools.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
