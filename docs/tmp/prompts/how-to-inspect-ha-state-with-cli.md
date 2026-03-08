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
- docs/src/how-to/inspect-ha-state-with-cli.md

[Page title]
- How to inspect HA state with pgtuskmasterctl

[Audience]
- An operator or developer who can reach a running API endpoint.

[User need]
- Query the current HA snapshot from the CLI and understand whether the request succeeded.

[Diataxis guidance]
- Action and only action.
- Stay focused on running the command and reading the immediate result.
- Link out for field definitions.

[Facts that are true]
- The CLI binary is pgtuskmasterctl.
- The default base URL is http://127.0.0.1:8080.
- The command path is: pgtuskmasterctl ha state
- The CLI accepts --base-url, --read-token, --admin-token, --timeout-ms, and --output.
- --output accepts json or text and defaults to json.
- Read requests use the read token when present, otherwise fall back to the admin token.
- The state command performs GET /ha/state.
- Success status is 200 OK.
- Text output renders key=value lines for cluster_name, scope, self_member_id, leader, switchover_requested_by, member_count, dcs_trust, ha_phase, ha_tick, ha_decision, and snapshot_sequence.
- Missing leader and switchover_requested_by render as <none> in text output.
- Transport or request-build failures exit with code 3.
- API status failures exit with code 4.
- Decode and output failures exit with code 5.

[Facts that must not be invented or changed]
- Do not promise a particular HA phase or leader value.
- Do not describe every response field in detail. Link to reference instead.

[Required structure]
- Goal sentence.
- Prerequisites.
- One sequence using default endpoint.
- One sequence using text output.
- One short troubleshooting section tied to exit behavior and HTTP reachability.
- Link-only related pages section.

[Related pages to link]
- ../reference/pgtuskmasterctl.md
- ../reference/http-api.md
- ../reference/ha-state-machine.md
- ../explanation/ha-decisions-and-actions.md

