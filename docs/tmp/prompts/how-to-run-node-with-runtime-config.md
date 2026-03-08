Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Draft a how-to guide.

[Page path]
- docs/src/how-to/run-a-node-with-a-runtime-config-file.md

[Page title]
- How to run pgtuskmaster with a runtime config file

[Audience]
- An operator or developer starting the daemon directly instead of through docker compose.

[User need]
- Start one node process with an explicit config file path and catch the common startup mistakes quickly.

[Diataxis guidance]
- Action and only action.
- Keep the guide bounded to launching the binary with a config file and checking the immediate startup result.

[Facts that are true]
- The daemon binary is pgtuskmaster.
- The binary accepts --config <PATH>.
- If --config is missing, the binary prints missing required `--config <PATH>` and exits with code 2.
- main delegates to run_node_from_config_path(config.as_path()) through a Tokio runtime.
- On runtime build failure or runtime::run_node_from_config_path error, the binary prints the error and exits with code 1.
- tests/cli_binary.rs verifies that a config missing config_version emits a message that includes: set config_version = "v2"
- tests/cli_binary.rs verifies that a config missing process.binaries emits an error mentioning `process.binaries`
- docker/configs/single/node-a/runtime.toml and docker/configs/cluster/node-a/runtime.toml are concrete v2 examples in the repo.

[Facts that must not be invented or changed]
- Do not provide a fake minimal config example.
- Do not claim the process daemonizes. The binary just runs until completion or error.

[Required structure]
- Goal sentence.
- Prerequisites.
- Steps to choose an existing v2 config file from the repo or your own file.
- Step to run pgtuskmaster --config <PATH>.
- Immediate verification and failure handling based on exit behavior and stderr.
- Link-only related pages section.

[Related pages to link]
- ../reference/pgtuskmaster.md
- ../reference/runtime-config.md
- ../reference/node-runtime.md

