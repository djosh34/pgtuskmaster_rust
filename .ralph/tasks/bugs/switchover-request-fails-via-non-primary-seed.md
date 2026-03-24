## Bug: Switchover request fails via non-primary seed config <status>not_started</status> <passes>false</passes> <priority>high</priority>

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
- [ ] I created a Red unit and/or integration test that captures the bug
- [ ] I made the test green by fixing
- [ ] I manually verified the bug, and created a new Red test if not working still
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
