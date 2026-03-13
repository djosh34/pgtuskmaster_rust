## Bug: HA compose should self-bootstrap with plain docker compose up <status>not_started</status> <passes>false</passes>

<description>
The HA docker assets under `tests/ha/givens/three_node_plain/compose.yml` do not currently behave like a self-contained docker-compose environment. A plain `docker compose up` for all services caused the three node containers to start before the seed-primary bootstrap sequence had been established, and each node exited early with DCS startup errors. The stack only became usable when it was started in the same staged order as the Rust HA harness:
- start `etcd`
- start `observer`
- start `node-b` as the seed primary
- wait for the seed primary to become authoritative
- start `node-a` and `node-c`

That contract currently lives in the test harness instead of in the docker environment itself. For local operability, demos, and source-backed manual validation, the compose stack should either:
- self-bootstrap correctly when the operator runs `docker compose up`, or
- be replaced by a dedicated docker-facing cluster asset that owns its own bootstrap orchestration instead of relying on hidden harness sequencing

Please explore and research the codebase first, then fix. The fix should move the startup assumptions out of the Rust HA harness and into the docker-facing path that a human operator actually invokes. Update the docker docs/examples so the supported manual cluster bring-up path works directly and deterministically.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
