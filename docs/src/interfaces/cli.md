# CLI Workflows

`pgtuskmasterctl` is the operator-friendly client for the Node API.

Common workflows:

- inspect current HA state and the selected HA decision
- submit switchover intent
- cancel pending switchover

```console
pgtuskmasterctl ha state
pgtuskmasterctl switchover --to <member-id>
pgtuskmasterctl switchover cancel
```

The CLI does not bypass API policy. It follows the same interface contract and security model.
