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
- docs/src/how-to/point-cli-at-a-non-default-api-endpoint.md

[Page title]
- How to point pgtuskmasterctl at a non-default API endpoint and token set

[Audience]
- An operator or developer whose API is not at the default local address or who needs token-protected access.

[User need]
- Run the CLI against another endpoint with the correct tokens and timeout.

[Diataxis guidance]
- Action and only action.
- Keep the page focused on selecting endpoint and token inputs.

[Facts that are true]
- The CLI defaults to --base-url http://127.0.0.1:8080.
- The CLI accepts --read-token and --admin-token.
- The CLI reads PGTUSKMASTER_READ_TOKEN and PGTUSKMASTER_ADMIN_TOKEN from the environment.
- Token strings are trimmed and blank strings become None.
- Read requests use the read token when present, otherwise fall back to the admin token.
- Admin requests use only the admin token.
- The CLI accepts --timeout-ms and defaults to 5000.
- parse_full_switchover_write_command in src/cli/args.rs shows a valid invocation with --base-url, --timeout-ms, --output text, and ha switchover request --requested-by node-a.
- The CLI can render output as json or text.

[Facts that must not be invented or changed]
- Do not invent shell aliases or config files for the CLI.
- Do not claim blank token arguments are valid configured tokens.

[Required structure]
- Goal sentence.
- Prerequisites.
- One sequence using flags only.
- One sequence using environment variables for tokens.
- One sequence that combines non-default base URL and timeout with a read request.
- Short troubleshooting section for missing admin token on write commands.
- Link-only related pages section.

[Related pages to link]
- ../reference/pgtuskmasterctl.md
- ../reference/http-api.md

