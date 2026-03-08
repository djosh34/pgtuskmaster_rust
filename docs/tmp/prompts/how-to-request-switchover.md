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
- docs/src/how-to/request-a-switchover.md

[Page title]
- How to request a switchover with pgtuskmasterctl

[Audience]
- An operator or developer who has admin access to a running cluster API.

[User need]
- Submit a switchover request and confirm the request was accepted.

[Diataxis guidance]
- Action and only action.
- Keep the guide bounded to issuing the request and checking the immediate outcome.
- Link out for HA semantics.

[Facts that are true]
- The CLI path is: pgtuskmasterctl ha switchover request --requested-by <VALUE>
- --requested-by is required.
- The CLI accepts --base-url, --admin-token, --timeout-ms, and --output.
- The switchover request performs POST /switchover.
- Admin requests use only the admin token.
- Success status is 202 Accepted.
- The request body is JSON: {"requested_by":"..."}.
- Successful CLI output can be rendered as JSON or text.
- AcceptedResponse contains accepted: bool.
- In HA test support, a successful request is followed by polling /ha/state until the stable primary changes.

[Facts that must not be invented or changed]
- Do not promise immediate leadership change on the first accepted request.
- Do not tell the user to write directly to DCS.
- Do not invent retry counts or failover timings.

[Required structure]
- Goal sentence.
- Prerequisites.
- Step sequence to send the request.
- Immediate verification step using accepted=true.
- Follow-up verification step using ha state polling.
- Short troubleshooting section for auth, transport, and non-changing primary.
- Link-only related pages section.

[Related pages to link]
- ../reference/pgtuskmasterctl.md
- ../reference/http-api.md
- ../reference/ha-state-machine.md
- ../explanation/ha-decisions-and-actions.md

