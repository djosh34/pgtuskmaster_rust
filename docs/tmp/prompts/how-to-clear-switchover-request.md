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
- docs/src/how-to/clear-a-pending-switchover.md

[Page title]
- How to clear a pending switchover request

[Audience]
- An operator or developer with admin API access.

[User need]
- Remove a previously requested switchover and confirm the clear request succeeded.

[Diataxis guidance]
- Action and only action.
- Stay on the single job of clearing a pending request.

[Facts that are true]
- The CLI path is: pgtuskmasterctl ha switchover clear
- The CLI accepts --base-url, --admin-token, --timeout-ms, and --output.
- The clear command performs DELETE /ha/switchover.
- Admin requests use only the admin token.
- Success status is 202 Accepted.
- The success payload is AcceptedResponse with accepted: bool.
- In API tests, DELETE /ha/switchover removes the /<scope>/switchover key and a later GET /ha/state reflects the cleared state.
- Text-mode HA state output renders switchover_requested_by=<none> when no request is present.

[Facts that must not be invented or changed]
- Do not claim the command changes primary roles by itself.
- Do not claim the clear command is safe for all cluster situations beyond removing the request.

[Required structure]
- Goal sentence.
- Prerequisites.
- Step sequence to send the clear command.
- Immediate verification step using accepted=true.
- Follow-up verification step using ha state and switchover_requested_by=<none>.
- Short troubleshooting section.
- Link-only related pages section.

[Related pages to link]
- ../reference/pgtuskmasterctl.md
- ../reference/http-api.md
- ../reference/ha-state-machine.md

