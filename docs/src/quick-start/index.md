# Quick Start

This path is now explicitly container-first. The shortest supported route to a real first run is the checked-in Docker Compose setup that starts `etcd` plus one or three `pgtuskmaster` nodes with tracked configs and file-backed Docker secrets.

You will do three things:

1. Prepare `.env.docker` and local secret files.
2. Start the single-node stack with `make docker-up`.
3. Validate the published API, debug, and PostgreSQL endpoints with the checked-in smoke flow.

After this section, go straight to the [Operator Guide](../operator/index.md) before you expose the API outside the local lab or translate the stack into a hardened production deployment.

The quick-start path is deliberately narrow. It is meant to prove that the repository-owned container assets, helper targets, and basic runtime surfaces all agree with each other on your machine. It is not meant to prove that your future production deployment is hardened, capacity-tested, or tuned for the topology you will eventually run.

Each stage answers a different question:

- **Prerequisites** answers "is my workstation and checkout capable of rendering the tracked lab stack correctly".
- **First Run** answers "can the repository-owned single-node deployment build, start, and expose the expected ports and routes".
- **Initial Validation** answers "do the externally visible signals agree that the node has become coherent enough to trust for further exploration".

If you only skim one thing before starting, skim those three questions. The quick start becomes much easier to debug when you know which proof failed. A missing secret file is not the same class of problem as a healthy container whose `/ha/state` never becomes coherent, and neither is the same as a PostgreSQL port that never becomes reachable from outside the container network.
