# Operator Guide

This section is the main operating manual for `pgtuskmaster`. It starts from the checked-in container path because that is the shortest supported route to a faithful deployment shape, but it is written to stay useful after you translate that shape into a more hardened environment.

Read by task, not by chapter loyalty:

| If you are trying to... | Start here |
| --- | --- |
| stand up the repo-owned deployment shape | [Container Deployment](./container-deployment.md) |
| understand or edit runtime settings | [Configuration Guide](./configuration.md) |
| reason about network exposure and topology assumptions | [Deployment and Topology](./deployment.md) |
| interpret API state, logs, and debug surfaces | [Observability and Day-2 Operations](./observability.md) |
| work backward from a symptom | [Troubleshooting by Symptom](./troubleshooting.md) |

Keep [System Lifecycle](../lifecycle/index.md) nearby while you read this section. The operator pages tell you what to inspect and change. The lifecycle pages explain why the node chooses phases such as bootstrap, replica, primary, rewinding, fencing, and fail-safe in the first place.

If you are in an incident, the fastest useful loop is usually:

1. read the symptom-oriented troubleshooting page
2. confirm the current state through observability surfaces
3. step into the lifecycle chapter that matches the current phase

That pattern keeps operational action tied to the current HA reasoning instead of to guesswork.
