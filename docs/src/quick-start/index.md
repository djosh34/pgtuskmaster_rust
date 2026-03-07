# Quick Start

This path is now explicitly container-first. The shortest supported route to a real first run is the checked-in Docker Compose setup that starts `etcd` plus one or three `pgtuskmaster` nodes with tracked configs and file-backed Docker secrets.

You will do three things:

1. Prepare `.env.docker` and local secret files.
2. Start the single-node stack with `make docker-up`.
3. Validate the published API, debug, and PostgreSQL endpoints with the checked-in smoke flow.

After this section, go straight to the [Operator Guide](../operator/index.md) before you expose the API outside the local lab or translate the stack into a hardened production deployment.
