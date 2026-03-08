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
- docs/src/how-to/boot-single-node-docker-stack.md

[Page title]
- How to boot the single-node docker stack and confirm it is healthy

[Audience]
- An operator or developer who wants to run the repo's single-node smoke environment.

[User need]
- Start the provided single-node stack and know when the stack is healthy enough to use.

[Diataxis guidance]
- This page must classify as action plus application.
- Action and only action.
- No explanations of architecture.
- No reference tables except tiny inline facts needed to complete the task.
- Link out instead of absorbing reference detail.
- Start and stop at a meaningful place.

[Facts that are true]
- The repo ships a smoke script at tools/docker/smoke-single.sh.
- The script requires curl and docker.
- The script uses docker/compose/docker-compose.single.yml.
- The script creates a temporary root with mktemp and writes a generated env file.
- The generated env file contains PGTM_SINGLE_API_PORT and PGTM_SINGLE_PG_PORT.
- The script creates temporary secret files for postgres-superuser.password, replicator.password, and rewinder.password.
- The script runs: docker compose --project-name "${PROJECT_NAME}" --env-file "${ENV_FILE}" -f "${COMPOSE_FILE}" up -d --build
- The script waits for HTTP 200 from http://127.0.0.1:${API_PORT}/ha/state
- The script waits for HTTP 200 from http://127.0.0.1:${API_PORT}/debug/verbose
- The script waits for a TCP listener on the published PostgreSQL port.
- The script waits for SQL readiness by running psql inside the node-a container against /var/lib/pgtuskmaster/socket and select 1.
- The script checks etcd health with etcdctl endpoint health inside the etcd container.
- The single-node compose file publishes host port ${PGTM_SINGLE_API_PORT} to container port 8080.
- The single-node compose file publishes host port ${PGTM_SINGLE_PG_PORT} to container port 5432.
- The node container mounts /etc/pgtuskmaster/runtime.toml from docker/configs/single/node-a/runtime.toml.
- That runtime config enables debug and sets api.listen_addr = "0.0.0.0:8080".
- The script cleans up the compose project and temp root on exit through a trap.

[Facts that must not be invented or changed]
- Do not invent fixed port numbers. The smoke script chooses free host ports dynamically.
- Do not claim the stack keeps running after the script exits. The trap tears it down.
- Do not claim TLS or auth are enabled in this docker smoke flow. The single-node runtime config disables both.

[Required structure]
- Short opening sentence stating the result.
- Prerequisites.
- Steps that tell the user to run the smoke script from the repo root.
- A section that tells the user what the script verifies before it reports success.
- A brief section telling the user where to look if the script exits early.
- A final "Related reference and explanation" section with links only.

[Related pages to link]
- ../reference/runtime-config.md
- ../reference/http-api.md
- ../reference/pgtuskmaster.md
- ../explanation/startup-versus-steady-state.md

