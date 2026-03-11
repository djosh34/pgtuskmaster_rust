## Bug: Greenfield mixed network fault can leave DCS-cut primary authoritative <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_mixed_network_faults_heal_converges` exposes a trustworthy mixed-fault behavior bug: cutting the current primary off from DCS while isolating a different node on observer API access can leave the original primary retaining authority instead of entering fail-safe or losing authority safely.

Observed on March 10, 2026 from:
- focused wrapper run: `cargo nextest run --profile ultra-long --target-dir /tmp/pgtuskmaster_rust-target --config build.incremental=false --test ha_mixed_network_faults_heal_converges`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for an authoritative stable primary
- selected a different non-primary node for API-only isolation
- cut the initial primary off from DCS
- isolated the chosen peer from observer API access

The failure happened on the first safety assertion for the mixed fault:
- step failure: `Then the node named "initial_primary" enters fail-safe or loses primary authority safely`
- observed error: `member \`node-b\` still held primary authority as \`node-b\``

This is trustworthy product evidence because the harness reached the intended mixed-fault state and the failure occurred on the product-side authority decision rather than on setup or observation. Explore and research the trust / fail-safe decision path first, then fix the product so a DCS-cut primary does not remain authoritative under this mixed-fault topology.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
