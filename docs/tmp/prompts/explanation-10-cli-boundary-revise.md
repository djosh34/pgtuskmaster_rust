Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise the supplied draft into a proper mdBook explanation page with the required heading and bounded sections.

[Output path]
- docs/src/explanation/cli-and-daemon-boundary.md

[Page title]
- # Why the CLI stays outside the control loop

[Existing draft to revise]
- docs/tmp/k2_outputs/explanation-10-cli-boundary.md

[Verified facts that are true]
- The pgtuskmaster binary accepts --config and delegates to runtime::run_node_from_config_path inside a Tokio runtime.
- The pgtuskmasterctl binary parses a clap CLI and delegates to cli::run, printing output on success and mapping errors to exit codes.
- CliApiClient constructs an HTTP client against a base URL, normalizes read and admin tokens, and exposes get_ha_state, delete_switchover, and post_switchover.
- Read operations use a read token when configured and fall back to the admin token when a read token is missing.
- Admin operations use the admin token.
- The policy test explicitly allows observation and switchover requests through supported API paths and forbids post-start internal steering.

[Required structure]
- Include the title heading.
- Explain the daemon-versus-client split.
- Explain why all control goes through supported HTTP surfaces.
- Explain what this means for automation and operational discipline.
