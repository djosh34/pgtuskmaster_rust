## Bug: Docker socket permission blocks greenfield HA harness runtime validation <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
Greenfield Docker HA cucumber scenarios cannot start in the current execution environment because `docker info` fails with:

`permission denied while trying to connect to the docker API at unix:///var/run/docker.sock`

This was detected while running `cargo nextest run --profile ultra-long --no-fail-fast --target-dir /tmp/pgtuskmaster_rust-target --config 'build.incremental=false' --test ha_api_path_isolation_preserves_primary` for story task 04. The test compiled and launched, but the first step failed before scenario setup because the harness validates Docker availability during `HarnessShared::initialize`.

Investigate the actual environment-level root cause first, then fix it in the most source-controlled way possible. That may mean correcting the local execution contract, adjusting how the harness discovers and invokes Docker, or documenting/enforcing an explicit prerequisite if the runtime truly cannot self-heal.
</description>

<blocked_by>
.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md
</blocked_by>
<blocked_by>
.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md
</blocked_by>
<blocked_by>
.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md
</blocked_by>
<blocked_by>
.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md
</blocked_by>

<acceptance_criteria>
- [ ] Explore and confirm why this environment cannot use the Docker socket even though the repo's HA harness requires Docker-backed execution.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] Because this blocks ultra-long HA wrappers: `make test-long` — passes cleanly
</acceptance_criteria>
