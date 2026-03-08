Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Draft a how-to guide.

[Page path]
- docs/src/how-to/read-ha-and-debug-endpoints.md

[Page title]
- How to read /ha/state and /debug/verbose during smoke validation

[Audience]
- An operator or developer validating a running smoke environment.

[User need]
- Query the two main observation endpoints directly with curl and know what success looks like.

[Diataxis guidance]
- Action and only action.
- Focus on observation steps and result checks.
- Link out for payload schemas.

[Facts that are true]
- tools/docker/smoke-single.sh waits for HTTP 200 from /ha/state and /debug/verbose on the single-node API port.
- tools/docker/smoke-cluster.sh waits for HTTP 200 from /ha/state and /debug/verbose on all three node API ports.
- GET /ha/state is a read route and returns 200 OK with HaStateResponse.
- GET /debug/verbose is a read route and returns 200 OK with JSON from build_verbose_payload.
- When cfg.debug.enabled is false, /debug/verbose returns 404 Not Found with body not found.
- When no snapshot subscriber is configured, /ha/state and /debug/verbose can return 503 Service Unavailable with body snapshot unavailable.
- The docker single-node and cluster runtime configs set debug.enabled = true.
- The smoke helper wait_for_ha_member_count checks member_count, ha_phase, and ha_decision in the /ha/state JSON payload.

[Facts that must not be invented or changed]
- Do not define the full JSON schema inline.
- Do not claim /debug/verbose is always available outside configs that enable debug.

[Required structure]
- Goal sentence.
- Prerequisites with curl and API base URL.
- A sequence for reading /ha/state.
- A sequence for reading /debug/verbose.
- A short section for interpreting common non-200 results during smoke validation.
- Link-only related pages section.

[Related pages to link]
- ../reference/http-api.md
- ../reference/debug-api.md
- ../reference/ha-state-machine.md

