# Quick Start

This path gets one node running with a real config, a reachable etcd cluster, and enough verification to prove the control loop is alive. It is not the full production guide. It is the shortest route to a first successful run that still reflects how the product actually works.

You will do three things:

1. Confirm the required binaries, directories, and secrets exist.
2. Start one node with an explicit `config_version = "v2"` config.
3. Verify the node API, logs, and DCS state before you move on.

After this section, go straight to the [Operator Guide](../operator/index.md) before you expose the API to other hosts or add more nodes.
