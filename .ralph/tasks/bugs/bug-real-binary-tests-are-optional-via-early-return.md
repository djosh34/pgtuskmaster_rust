---
## Bug: Real Binary Tests Become Optional Via Early Return <status>not_started</status> <passes>false</passes>

<description>
Several real-binary test paths silently return `Ok(())` when required binaries are not discovered (for example `None => return Ok(())`).
This makes critical runtime coverage optional and can mask regressions in HA/bootstrap/process behavior.

Explore and research the full codebase first, then implement a fix so real-binary tests are enforced instead of being skipped by default.
The solution should preserve clear error messages about missing prerequisites and keep CI/local workflows deterministic.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
