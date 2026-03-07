# Prerequisites

The default first-run path is the checked-in Docker Compose stack. You do not need a host-level PostgreSQL or etcd install for that path, but you do need Docker plus three readable local secret files.

Required on the host:

- Docker Engine with the `docker compose` v2 plugin
- enough disk space for one or three PostgreSQL data volumes, depending on which stack you plan to start
- free local ports for the API and PostgreSQL mappings you plan to publish (for `make docker-up` this comes from `.env.docker`; the smoke targets pick free ports automatically)
- a writable checkout so the helper targets can build images and write temporary smoke artifacts

Before the first Compose launch:

1. Create `.env.docker` from the tracked `.env.docker.example` file at the repository root.
2. Ensure the three secret env vars point at readable, non-empty files.
   - For a local lab you may point at the tracked `docker/secrets/*.example` placeholders.
   - For anything hardened, use real secret files outside the repository checkout.
   - The Compose files live under `docker/compose/`, so relative secret paths in the env file are interpreted from that directory. The tracked `.env.docker.example` uses `../secrets/...` so the default lab wiring resolves correctly without needing absolute paths.
3. Write a non-empty value into each referenced file (even in the lab):
   - `PGTM_SECRET_SUPERUSER_FILE`
   - `PGTM_SECRET_REPLICATOR_FILE`
   - `PGTM_SECRET_REWINDER_FILE`
4. Keep real secret files out of git. The repository only tracks placeholder `docker/secrets/*.example` files.

For the checked-in local lab, those files primarily satisfy the runtime's file-backed secret contract while PostgreSQL network auth stays trust-based inside the private Compose bridge. For a hardened deployment, replace the placeholder values with strong real passwords and lock down `pg_hba`.

The checked-in Compose assets use:

- `PGTUSKMASTER_IMAGE` for the node image tag
- `ETCD_IMAGE` for the etcd image tag
- `PGTM_SINGLE_*` ports for the single-node stack
- `PGTM_CLUSTER_NODE_*_*` ports for the three-node cluster stack

If you want the default quick-start validation path to work before you do a full `up`, run:

```console
docker compose --env-file .env.docker -f docker/compose/docker-compose.single.yml config
docker compose --env-file .env.docker -f docker/compose/docker-compose.cluster.yml config
```

Manual or non-container deployment is still supported as an advanced path, but it is no longer the default first-run workflow. Use the Compose flow first unless you are deliberately building a host-native installation.

## Why each prerequisite matters

### Docker Engine and the `docker compose` v2 plugin

The checked-in quick start uses `make docker-up`, `make docker-smoke-single`, and `make docker-smoke-cluster`, all of which assume a working Docker daemon plus the modern Compose plugin. If either piece is missing, you are blocked before the runtime even starts. More importantly, a broken Docker setup can create misleading symptoms: configuration may render fine while the image build fails, containers may be created while published ports never bind, or the daemon may be unreachable even though the CLI binary exists on `PATH`.

The safest early check is `docker info` followed by `docker compose version`. If those fail, fix Docker first. Do not burn time debugging `pgtuskmaster` behavior in a host environment that cannot reliably start the lab topology.

### Free local ports

The Compose files publish the API and PostgreSQL ports declared in `.env.docker` when you use `make docker-up` (or an equivalent `docker compose ... up`). A port collision does not mean the cluster logic is wrong; it means your host cannot expose the path the validation steps are about to use.

That matters because the quick start proof is specifically an external-observer proof: once the stack is up, you validate via `http://127.0.0.1:<port>/ha/state`, `http://127.0.0.1:<port>/debug/verbose`, and the published PostgreSQL TCP port.

The smoke targets are intentionally different. `make docker-smoke-single` and `make docker-smoke-cluster` generate a temporary env file and pick random free ports so they can validate a fresh isolated Compose project without depending on your `.env.docker` port choices. That makes smoke runs robust against `.env.docker` collisions, but it also means a passing smoke run does not prove that the specific ports in `.env.docker` are available on your host.

If you already run local PostgreSQL, reverse proxies, or other services on nearby ports, edit `.env.docker` before first launch. A quick start that only works after ad-hoc port surgery in the middle of the flow is harder to interpret and easier to misread.

### Readable secret files

The tracked example environment points at placeholder files under `docker/secrets/*.example`, but the real contract is "three readable, non-empty files exist at the configured paths". Compose loads those files as Docker secrets, and the runtime expects them to exist consistently. If a path is wrong, Compose rendering can fail before container creation. If a file exists but is empty or unreadable, startup can proceed far enough to be confusing and then fail when PostgreSQL auth-related setup or runtime secret access occurs.

For the default lab path, these values mainly satisfy the file-backed secret contract while PostgreSQL network auth remains trust-based inside the private bridge network. That should not tempt you to treat them as optional. The runtime shape you are proving still depends on those files being present, and later production hardening depends on replacing placeholder values with real credentials and tighter access controls.

### Writable checkout and enough disk space

The quick start builds the production image, creates local volumes, and writes temporary smoke artifacts. If the checkout is read-only, the build can fail in ways that look unrelated to HA behavior. If the host is low on space, PostgreSQL data volumes or image layers can fail late, after some services have already started. That class of failure is especially easy to misread because a partial stack may expose the API before PostgreSQL is actually usable.

Treat workspace writability and disk space as correctness prerequisites, not convenience prerequisites. The point of the quick start is to separate environment readiness from cluster semantics. A cramped or read-only host collapses those questions together.

## Common misreads before first launch

### "The example secret paths mean I can skip creating real files"

You cannot skip having secret files. The Compose path expects three readable, non-empty files and mounts them as Docker secrets.

For the local lab you may point `.env.docker` at the tracked `docker/secrets/*.example` placeholders, but treat them as insecure and replace them with strong real secrets for anything beyond an isolated demo environment.

### "If `docker compose config` renders, the stack is basically ready"

Rendering proves that variable substitution, file references, and Compose syntax are coherent. It does not prove the Docker daemon is healthy, the image build will succeed, the containers will pass health checks, or the published ports will be reachable. Treat rendering as an early fail-fast step, not as full validation.

### "A host-native install is equivalent to the quick start"

Not for this book. Host-native deployment may be supported as an advanced path, but it bypasses the exact container assets and smoke coverage that the quick start is written around. If you skip the checked-in Compose route first, you lose the easiest baseline for deciding whether a later deployment issue comes from the project or from your translation layer.

## Symptom-level consequences when prerequisites are wrong

- Missing or incorrect `.env.docker`: Compose render fails or uses surprising port or secret values.
- Missing secret file: Compose config or container startup fails before a clean runtime proof exists.
- Docker daemon unavailable: helper targets fail immediately even though the repository content is fine.
- Port collision: `/ha/state` or PostgreSQL checks fail from the host despite containers existing.
- Unwritable checkout or exhausted disk: image builds, volumes, or smoke artifacts fail after partial startup, creating noisy but misleading symptoms.

If you hit any of those, stop and correct the prerequisite rather than continuing deeper into the quick-start flow. The later lifecycle and operator explanations assume the environment proof is already sound.
