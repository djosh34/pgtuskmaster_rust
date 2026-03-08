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
- docs/src/how-to/boot-three-node-docker-cluster.md

[Page title]
- How to boot the three-node docker cluster and verify one primary plus two replicas

[Audience]
- An operator or developer who wants to run the repo's multi-node smoke environment.

[User need]
- Start the provided three-node cluster and confirm it converges to a healthy topology.

[Diataxis guidance]
- This page must stay goal-oriented and procedural.
- Action and only action.
- No architecture explanation and no feature tour.

[Facts that are true]
- The repo ships tools/docker/smoke-cluster.sh.
- The script requires curl and docker.
- The script uses docker/compose/docker-compose.cluster.yml.
- The compose file defines etcd plus node-a, node-b, and node-c.
- The script generates an env file with host API and PostgreSQL ports for each node.
- The script runs docker compose up -d --build with the generated env file and project name.
- The script waits for HTTP 200 from /ha/state on node-a, node-b, and node-c.
- The script waits for HTTP 200 from /debug/verbose on node-a, node-b, and node-c.
- The script waits for published PostgreSQL TCP ports for node-a, node-b, and node-c.
- The script waits for /ha/state to report member_count >= 3 on all three nodes.
- The script waits for SQL readiness on node-a, node-b, and node-c by running psql inside the containers.
- The script checks replication roles by running select pg_is_in_recovery() inside each node container.
- The script treats the cluster as healthy only when exactly one node reports f and two nodes report t from pg_is_in_recovery().
- The script checks etcd health inside the etcd container.
- The cluster runtime config for node-a uses cluster.name = "docker-cluster", member_id = "node-a", dcs.scope = "docker-cluster", debug.enabled = true, and api.listen_addr = "0.0.0.0:8080".
- The script tears the environment down on exit via a trap.

[Facts that must not be invented or changed]
- Do not invent static host ports.
- Do not claim the smoke environment persists after the script exits.
- Do not claim a specific node is always primary. The health check only requires one primary and two replicas.

[Required structure]
- Goal statement.
- Prerequisites.
- Steps to run the cluster smoke script.
- Steps to interpret a healthy outcome, including API, member count, and replication role checks.
- A short section for what to inspect if convergence stalls.
- Link-only related pages section.

[Related pages to link]
- ../reference/http-api.md
- ../reference/runtime-config.md
- ../reference/ha-state-machine.md
- ../explanation/ha-decisions-and-actions.md
- ../explanation/dcs-trust-and-coordination.md

