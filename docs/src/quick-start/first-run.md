# First Run

This flow is a practical first launch. It validates the startup planner path, API availability, and basic DCS coordination behavior.

## 1. Prepare configuration

Use the recommended baseline in [Operator Guide / Configuration Guide](../operator/configuration.md). For first run, keep the scope small and explicit.

## 2. Start dependencies

Bring up etcd and verify reachability from the node host.

## 3. Start the node

Start `pgtuskmaster` with your config file.

```console
pgtuskmaster --config /path/to/runtime.toml
```

## 4. Verify API status

Query HA state from the node API or CLI.

```console
pgtuskmasterctl ha state
```

You should see a coherent phase and trust posture, not an empty or unreachable response.

## Why this matters

A first run is successful when the control loop is alive, not only when a process exists. API state confirms that workers are observing and publishing state instead of failing silently.
