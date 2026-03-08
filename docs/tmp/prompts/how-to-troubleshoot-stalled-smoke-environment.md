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
- docs/src/how-to/troubleshoot-a-smoke-environment-that-stalls.md

[Page title]
- How to troubleshoot a smoke environment that never reaches the expected member count or replication roles

[Audience]
- An operator or developer whose smoke-single or smoke-cluster run does not converge.

[User need]
- Use the repo's own readiness checks to isolate which stage is failing.

[Diataxis guidance]
- Action and only action.
- Keep troubleshooting anchored to repo-backed observables.
- No long explanations of internals.

[Facts that are true]
- The smoke scripts fail on missing curl or docker before starting any containers.
- The smoke scripts wait for /ha/state and /debug/verbose before later checks.
- The cluster smoke script waits for member_count >= 3 on all nodes.
- The cluster smoke script waits for exactly one primary and two replicas by checking pg_is_in_recovery() inside each node container.
- The smoke scripts check SQL readiness with psql inside each node container.
- The smoke scripts check etcd health with etcdctl endpoint health inside the etcd container.
- The helper functions print timeout messages such as timed out waiting for <label> at <address>, timed out waiting for SQL readiness on service <service>, and timed out waiting for 3 members at <url>.
- The docker runtimes enable /debug/verbose.
- /ha/state can return snapshot unavailable when no snapshot subscriber is configured.
- /debug/verbose can return snapshot unavailable when no snapshot subscriber is configured.
- In the cluster helper, /ha/state polling is the canonical post-start observation path.

[Facts that must not be invented or changed]
- Do not tell the user to mutate DCS directly.
- Do not invent a container log command that is not part of docker compose.
- Do not guarantee one root cause from a timeout alone.

[Required structure]
- Goal sentence.
- Prerequisites.
- A staged troubleshooting sequence:
- first command availability
- then API reachability
- then member count
- then PostgreSQL port and SQL readiness
- then replication role convergence
- then etcd health
- Include concrete checks using curl, docker compose exec, and the same SQL query used by the smoke script where applicable.
- End with link-only related pages.

[Related pages to link]
- ../reference/http-api.md
- ../reference/ha-state-machine.md
- ../reference/runtime-config.md
- ../explanation/ha-decisions-and-actions.md
