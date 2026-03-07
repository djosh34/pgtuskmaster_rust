# First Run

Use a small, explicit config for the first launch. Keep the scope narrow: one node, one etcd cluster, one PostgreSQL data directory, and no speculative deployment extras.

1. Prepare the runtime config.

Use the production or local-only examples in [Configuration Guide](../operator/configuration.md). For the first run, keep every path absolute and keep `cluster.member_id` and `dcs.scope` consistent with the etcd namespace you intend to use.

2. Start etcd and confirm the node can reach it.

The node will not guess around a broken DCS path. If etcd is unreachable, fix that first.

3. Start the node.

```console
pgtuskmaster --config /path/to/runtime.toml
```

4. Query the node state.

If you are using the local-only API example, the CLI works against the default runtime API address without extra flags:

```console
pgtuskmasterctl ha state
```

If you exposed the API on another host or enabled HTTPS and tokens, point the CLI at that endpoint instead:

```console
pgtuskmasterctl \
  --base-url https://node-a.example.com:8080 \
  --read-token "$PGTUSKMASTER_READ_TOKEN" \
  ha state
```

The first successful run is not just “the process stayed up.” It is “the node published a coherent `/ha/state`, the logs explain the chosen startup path, and the DCS scope reflects the same member identity you configured.”
