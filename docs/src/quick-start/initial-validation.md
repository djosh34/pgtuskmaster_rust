# Initial Validation

Treat the first launch as incomplete until you can prove the checked-in container stack is reachable from outside the containers.

Run through this checklist:

- `make docker-smoke-single` passes cleanly.
- `GET /ha/state` on `http://127.0.0.1:${PGTM_SINGLE_API_PORT}` returns a coherent JSON payload for `node-a`.
- `GET /debug/verbose` on the same published API port succeeds, proving the debug routes ride the API listener instead of a second debug socket.
- the published PostgreSQL port at `PGTM_SINGLE_PG_PORT` accepts TCP connections
- `etcdctl endpoint health` succeeds inside the `etcd` container
- PostgreSQL accepts a local control query inside the node container

What good looks like:

- a brand new single-node lab usually converges to a primary-oriented state
- the member identity in `/ha/state` matches `node-a`
- the API and PostgreSQL ports exposed by Compose are the only externally published service ports
- the logs explain the chosen startup path instead of leaving you to infer behavior from container liveness alone

When you are ready for the full multi-node lab, run:

```console
make docker-smoke-cluster
```

That smoke flow brings up `etcd`, `node-a`, `node-b`, and `node-c`, verifies `/ha/state`, `/debug/verbose`, and published PostgreSQL ports for every node, then tears the stack down automatically.

## What each validation signal proves

### `make docker-smoke-single`

This is the strongest repository-owned proof because it does not just test one route. The script creates a temporary environment file, generates temporary secret files, builds and starts the single-node stack, waits for `/ha/state`, waits for `/debug/verbose`, waits for the published PostgreSQL TCP port, checks SQL readiness inside `node-a`, and verifies etcd health. When this passes, you know the checked-in container assets and the basic runtime surfaces agree end to end in a fresh environment.

If it fails, read the first failing probe rather than the last line. A timeout on `/ha/state` means something different from a timeout on the PostgreSQL TCP port, and both mean something different from an etcd health failure. The smoke target is useful because it preserves that ordering of evidence.

### `GET /ha/state`

This route confirms more than mere HTTP reachability. It exposes the node's current cluster name, scope, self member id, current trust posture, HA phase, current decision, and snapshot sequence. A healthy response is therefore one that is both reachable and coherent. The member identity should match the node you started, the cluster and scope should match the active config, and the phase and decision pair should describe a believable local state rather than contradictory fragments.

Suspicious but informative outputs include:

- a valid response whose member id does not match the expected node
- a phase that never moves past an early waiting state
- trust or leader data that remains absent longer than the rest of the stack suggests it should

Those are not all equal in severity. Some indicate normal convergence delay, while others indicate configuration drift or coordination visibility problems.

### `GET /debug/verbose`

In the checked-in quick-start path, this route proves that the debug payload is available through the same published API listener as `/ha/state`. That matters because the lab is intentionally easy to inspect. If `/ha/state` works and `/debug/verbose` does not, the node may still be operating, but your debugging surface no longer matches the quick-start assumptions. Treat that as a configuration or exposure mismatch, not as a total service failure.

### Published PostgreSQL port

The PostgreSQL TCP probe proves that a client outside the Compose network can at least reach the database port the stack claims to publish. It does not, by itself, prove that the node is in the right HA role or that every auth path is production-ready. It does prove that the container-level database path is exposed in a way that matches the Compose contract.

If this check fails while the API checks pass, look first at PostgreSQL startup, container port publication, and host conflicts. The HA control plane can be healthy while the data plane is still not externally reachable.

### etcd health and local SQL readiness

These inside-the-container checks complete the picture. etcd health confirms that the coordination service is actually functioning, not merely that the container exists. SQL readiness confirms that PostgreSQL inside the node container can accept a local control query, which helps separate "published host port failed" from "database never became usable at all".

Taken together, these checks tell you whether you have:

- a reachable control surface
- a reachable data surface
- a functioning coordination backend
- a locally usable PostgreSQL process

That combination is the real definition of "the quick-start lab works".

## How to interpret healthy versus risky outcomes

On a brand new single-node lab, convergence toward a primary-oriented state is usually the healthy outcome because there is no competing node to follow. Healthy output is not just "some JSON came back". Healthy output means the signals agree with each other: API responses name the expected node, PostgreSQL is reachable, and the logs explain a path that fits the observed phase.

Risky outcomes tend to show up as disagreement across surfaces:

- `/ha/state` reachable but PostgreSQL port unreachable
- debug route missing from the expected API port
- etcd unhealthy while the node appears superficially alive
- a member count or leadership view that does not fit the topology you actually launched

When signals disagree, stop and classify the disagreement before you change anything. Mixed evidence is exactly where unsafe HA reasoning starts.

## When to move on

Move on to the Operator Guide only after you can explain, in plain language, what each of the core surfaces is currently proving:

- the API tells you how the node sees the cluster
- the debug route tells you whether the richer lab inspection surface is exposed
- the PostgreSQL port tells you whether the data path is reachable from outside the containers
- etcd health tells you whether shared coordination is alive
- SQL readiness tells you whether the local PostgreSQL process is actually usable

Once you can make those distinctions, the deeper operator and lifecycle chapters become much easier to apply correctly.
