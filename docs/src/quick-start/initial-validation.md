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
