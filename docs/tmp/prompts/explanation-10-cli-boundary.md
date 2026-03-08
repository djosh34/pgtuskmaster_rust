Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Draft a new explanation page.

[Output path]
- docs/src/explanation/cli-and-daemon-boundary.md

[Page title]
- # Why the CLI stays outside the control loop

[Audience]
- Readers trying to understand how pgtuskmasterctl relates to the node process and why it is intentionally narrow.

[User need]
- Understand why the CLI is an external client of the HTTP API instead of a local control backdoor into runtime internals.

[mdBook context]
- Link naturally to pgtuskmaster, pgtuskmasterctl, HTTP API, and API/debug explanation pages.

[Diataxis guidance]
- Explanation only.

[Verified facts that are true]
- The pgtuskmaster binary accepts --config and delegates to runtime::run_node_from_config_path inside a Tokio runtime.
- The pgtuskmasterctl binary parses a clap CLI and delegates to cli::run, printing output on success and mapping errors to exit codes.
- CliApiClient constructs an HTTP client against a base URL, normalizes read and admin tokens, and exposes get_ha_state, delete_switchover, and post_switchover.
- Read operations use a read token when configured and fall back to the admin token when a read token is missing.
- Admin operations use the admin token.
- The e2e policy explicitly allows observation and switchover requests through supported API paths and forbids post-start internal steering.

[Relevant repo grounding]
- src/bin/pgtuskmaster.rs
- src/bin/pgtuskmasterctl.rs
- src/cli/client.rs
- tests/policy_e2e_api_only.rs

[Design tensions to explain]
- Why the CLI is not allowed to bypass the daemon through internal hooks.
- Why role-separated tokens matter even for a small command surface.
- Why the supported control surface is narrow.

[Required structure]
- Explain the daemon-versus-client split.
- Explain why all control goes through supported HTTP surfaces.
- Explain what this means for automation and operational discipline.

[Facts that must not be invented or changed]
- Do not claim the CLI can manipulate DCS directly.
- Do not claim the CLI embeds HA logic.
