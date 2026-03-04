---
## Bug: BDD HTTP tests false-pass via fragile status and read patterns <status>not_started</status> <passes>false</passes>

<description>
BDD HTTP contract tests use weak status matching and response-read behavior that can hide protocol/handler regressions or induce hangs.

Detected during review of plan section 2.4 against:
- tests/bdd_api_http.rs:247-250, 298-301, 317-320, 336-339, 355-358, 407-410, 434-437, 455-458, 498-501, 539-542, 577, 598-601
  (status assertions use `contains("...")` instead of exact status code checks)
- tests/bdd_api_http.rs:241-245, 293-296, 312-315, 330-334, 350-353, 402-405, 429-432, 450-453, 493-496, 534-537, 573-575, 593-596
  (response reads rely on `read_to_end`, which depends on connection-close semantics)

Ask the fixing agent to explore and research the codebase first, then implement robust HTTP response assertions.

Expected direction:
- Parse status code as an exact numeric value and assert exact expectations.
- Replace connection-close-dependent `read_to_end` flow with bounded HTTP response reads that honor headers/content length (or a tested helper used by all cases).
- Keep tests black-box and contract-oriented; avoid overfitting to current low-level socket behavior.
- Add one negative test proving a wrong status code cannot satisfy assertions.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
