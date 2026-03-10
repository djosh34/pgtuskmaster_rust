## Bug: Greenfield HA Node Image Relies On PGSSLROOTCERT Env Hack <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>

<description>
The greenfield HA node fixture still bakes `ENV PGSSLROOTCERT=/etc/pgtuskmaster/tls/ca.crt` into the node image instead of carrying the PostgreSQL client CA path through runtime-managed configuration.

Detection context on March 10, 2026:
- the user explicitly called out the line in `cucumber_tests/ha/givens/three_node_plain/docker_files/node.Dockerfile`
- the line is still present after the harness cleanup work
- the current runtime-side remote verify-full paths still depend on ambient libpq environment rather than an explicit runtime/rendered conninfo CA path

The executor should explore the codebase first, then fix the underlying runtime/config design rather than moving the same environment variable to another layer.

Important code areas already implicated:
- `cucumber_tests/ha/givens/three_node_plain/docker_files/node.Dockerfile`
- `src/pginfo/conninfo.rs`
- `src/ha/source_conn.rs`
- `src/postgres_managed_conf.rs`
- `src/runtime/node.rs`

The intended direction is:
- remote PostgreSQL verify-full connections used by runtime-managed HA flows should carry the CA path through explicit config/rendered conninfo
- the fixture node image should stop depending on `PGSSLROOTCERT`
- docs/examples should reflect the runtime-configured path instead of an image-level environment hack
</description>

<acceptance_criteria>
- [ ] runtime-managed remote PostgreSQL connections have an explicit, source-backed CA-path configuration path instead of relying on ambient `PGSSLROOTCERT`
- [ ] `cucumber_tests/ha/givens/three_node_plain/docker_files/node.Dockerfile` no longer sets `ENV PGSSLROOTCERT=/etc/pgtuskmaster/tls/ca.crt`
- [ ] the greenfield HA fixture still runs with verify-full semantics after the env hack is removed
- [ ] docs/examples are updated anywhere they would otherwise imply the env-hack approach
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] if the fix touches ultra-long HA behavior or its selection: `make test-long` — passes cleanly
</acceptance_criteria>
