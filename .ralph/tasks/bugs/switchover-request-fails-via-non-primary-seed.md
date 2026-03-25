## Bug: Switchover request fails via non-primary seed config <status>completed</status> <passes>true</passes> <priority>high</priority>

<description>
`pgtm switchover request` currently posts to the configured seed node API even after it has already read cluster state proving that another member is the authoritative primary.

This was reproduced in the shipped Docker walkthrough on 2026-03-24:

- `pgtm -c docker/pgtm.toml status` succeeded from the host and showed `node-c` as primary.
- `pgtm -c docker/pgtm.toml switchover request --switchover-to node-b` failed with:
  `resolution error: cannot request switchover via 'node-a': seed node is not the authoritative primary`

That means the canonical host-side operator config is sufficient for reads but not for planned failover actions once leadership moves away from its fixed seed.

This is a real operator-product bug, not just a docs gap. The CLI already has authoritative cluster state in hand, but the write path cannot continue unless the operator manually swaps to another seed config whose `api.base_url` points at the current primary.
</description>

<mandatory_red_green_tdd>
Use Red-Green TDD to solve the problem.
You must make ONE test, and then make ONE test green at the time.

Then verify if bug still holds. If yes, create new Red test, and continue with Red-Green TDD until it does work.
</mandatory_red_green_tdd>

<acceptance_criteria>
- [x] I created a Red unit and/or integration test that captures the bug
- [x] I made the test green by fixing
- [x] I manually verified the bug, and created a new Red test if not working still
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<plan>
- [x] Finish the type boundary first so write routing is shared instead of seed-local: promote operator API advertisement into a real shared member shape, preferably alongside the existing network route types in `src/state/net.rs` / `src/dcs/state.rs`, and reuse that shape everywhere instead of threading raw URL strings through CLI-only helpers.
- [x] Make `pgtm.api.advertised_url` real across config ingestion instead of rejecting it: thread it through `load_operator_config`, add the runtime-side validated field that node startup can publish, and remove the current unsupported-placeholder branch so runtime docs and standalone operator docs share one operator API advertisement story.
- [x] Publish the authoritative member's operator-visible API route in DCS member state and update the shipped/runtime fixtures that need distinct external URLs (`docker/node-a.toml`, `docker/node-b.toml`, `docker/node-c.toml`, and any HA/generated observer config paths) so host-side operator flows can reach `18081` / `18082` / `18083` instead of only the seed's fixed URL.
- [x] Add one Red CLI-facing test that captures the actual bug: fetch state from a seed mock, return a quorum snapshot whose authoritative primary is another member with an advertised operator API URL, and assert that `pgtm switchover request` still does `GET /state` against the seed but sends `POST /switchover` to the authoritative primary's advertised URL.
- [x] Make that one test green by collapsing switchover validation and routing into one authoritative-primary resolution path in the CLI: reuse the seed-derived auth/TLS/timeout settings, swap only the API destination to the authoritative member advertisement, and fail with a precise resolution error if quorum exists but the authoritative member does not advertise an operator API route.
- [x] Manually re-run the shipped Docker walkthrough after the type/routing change. If the operator-visible flow still fails anywhere, stop and add the next Red test for the exact remaining failure before changing more behavior.
- [x] After the manual verification is green, clean up any now-dead seed-only helpers or duplicated routing branches so `status`/`primary`/`replicas` keep their read behavior while switchover writes use the shared authoritative routing boundary instead of another special case.
- [x] Run the required validation gates in repo order only after the design still looks correct: `make check`, `make lint`, `make test`, `make test-long`. If execution shows the advertised-route ADT or routing boundary is still wrong, switch this task back to `TO BE VERIFIED`, explain the exact mismatch here, and stop immediately.
</plan>
