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
- docs/src/how-to/confirm-published-postgresql-readiness.md

[Page title]
- How to confirm published PostgreSQL readiness in the docker environments

[Audience]
- An operator or developer validating the repo's single-node or cluster docker environment.

[User need]
- Confirm that PostgreSQL is reachable on the published host port and actually answering SQL inside the containers.

[Diataxis guidance]
- Action and only action.
- Stay on readiness checks, not general PostgreSQL administration.

[Facts that are true]
- tools/docker/common.sh provides wait_for_tcp_port(host, port, label, timeout_secs).
- wait_for_tcp_port uses bash /dev/tcp to check whether the host port is listening.
- tools/docker/common.sh provides wait_for_sql_ready(compose_file, env_file, project_name, service_name, password, timeout_secs).
- wait_for_sql_ready runs docker compose exec -T <service> /bin/bash -lc "/usr/lib/postgresql/16/bin/psql -h /var/lib/pgtuskmaster/socket -U postgres -d postgres -Atqc 'select 1'"
- The smoke-single script waits for the published PostgreSQL port, then waits for SQL readiness on node-a.
- The smoke-cluster script waits for the published PostgreSQL port on node-a, node-b, and node-c, then waits for SQL readiness on each node.
- The compose files publish container port 5432 onto generated host ports from the env file.
- The runtime configs place the PostgreSQL socket at /var/lib/pgtuskmaster/socket.

[Facts that must not be invented or changed]
- Do not tell the user to expect the generated host port to be 5432.
- Do not claim the readiness check uses network SQL from the host. The SQL readiness helper runs inside the container over the socket.

[Required structure]
- Goal sentence.
- Prerequisites.
- Steps to identify the published port from the generated env file.
- Steps to confirm the port is listening.
- Steps to confirm SQL readiness inside the container.
- Short section for what a mismatch means if the port is open but SQL is not ready.
- Link-only related pages section.

[Related pages to link]
- ../reference/runtime-config.md
- ../reference/managed-postgres.md
- ../reference/node-runtime.md

