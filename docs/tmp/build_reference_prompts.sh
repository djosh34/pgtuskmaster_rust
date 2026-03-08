#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PROMPT_DIR="$ROOT_DIR/docs/tmp/prompts"

mkdir -p "$PROMPT_DIR"

write_common_header() {
  local prompt_path="$1"
  local page_path="$2"
  local page_goal="$3"
  local required_sections="$4"

  cat >"$prompt_path" <<EOF_PROMPT
Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise an existing reference page so it stays strictly in Diataxis reference form.

[Page path]
- $page_path

[Page goal]
- $page_goal

[Audience]
- Operators and contributors who need accurate repo-backed facts while working with pgtuskmaster.

[User need]
- Consult the machinery surface, data model, constraints, constants, and behavior without being taught procedures or background explanations.

[mdBook context]
- This is an mdBook page under docs/src/reference/.
- Keep headings and lists suitable for mdBook.
- Do not add verification notes, scratch notes, or commentary about how the page was produced.

[Diataxis guidance]
- This page must stay in the reference quadrant: cognition plus application.
- Describe and only describe.
- Structure the page to mirror the machinery, not a guessed workflow.
- Use neutral, technical language.
- Examples are allowed only when they illustrate the surface concisely.
- Do not include step-by-step operations, recommendations, rationale, or explanations of why the design exists.
- If action or explanation seems necessary, keep the page neutral and mention the boundary without turning the page into a how-to or explanation article.

[Required structure]
$required_sections

[Output requirements]
- Preserve or improve the title so it matches the machinery.
- Prefer compact tables where they clarify enums, fields, constants, routes, or error variants.
- Include only facts supported by the supplied source excerpts.
- If the current page contains unsupported claims, remove or correct them rather than hedging.

[Existing page to revise]

EOF_PROMPT
  cat "$ROOT_DIR/$page_path" >>"$prompt_path"
  cat >>"$prompt_path" <<'EOF_PROMPT'

[Repo facts and source excerpts]

EOF_PROMPT
}

append_file() {
  local prompt_path="$1"
  local repo_path="$2"

  {
    printf -- '--- BEGIN FILE: %s ---\n' "$repo_path"
    cat "$ROOT_DIR/$repo_path"
    printf -- '\n--- END FILE: %s ---\n\n' "$repo_path"
  } >>"$prompt_path"
}

build_prompt() {
  local slug="$1"
  local page_path="$2"
  local page_goal="$3"
  local required_sections="$4"
  shift 4
  local prompt_path="$PROMPT_DIR/$slug.prompt.md"

  write_common_header "$prompt_path" "$page_path" "$page_goal" "$required_sections"
  for repo_path in "$@"; do
    append_file "$prompt_path" "$repo_path"
  done
}

build_prompt \
  "runtime-config" \
  "docs/src/reference/runtime-config.md" \
  "Reference the runtime configuration loader, normalized schema, defaults, and validation boundaries exposed by the config modules." \
  "- Overview\n- Module surface\n- Load pipeline and version handling\n- Normalized runtime config structure\n- Defaulted fields\n- Validation rules and invariants\n- Related bundled artifacts if directly sourced from the repo" \
  "src/config/mod.rs" \
  "src/config/schema.rs" \
  "src/config/defaults.rs" \
  "src/config/parser.rs" \
  "tests/cli_binary.rs"

build_prompt \
  "dcs" \
  "docs/src/reference/dcs.md" \
  "Reference the DCS keyspace, cached state, trust model, watch refresh behavior, worker loop, and etcd-backed store implementation." \
  "- Overview\n- Module surface\n- Keyspace and record types\n- Cache and trust model\n- Store and watch surface\n- Worker loop\n- Etcd-backed implementation constants and reconnect behavior" \
  "src/dcs/mod.rs" \
  "src/dcs/keys.rs" \
  "src/dcs/state.rs" \
  "src/dcs/store.rs" \
  "src/dcs/worker.rs" \
  "src/dcs/etcd_store.rs"

build_prompt \
  "http-api" \
  "docs/src/reference/http-api.md" \
  "Reference the HTTP API worker, transport handling, auth rules, supported routes, and response contract." \
  "- Overview\n- Worker and transport behavior\n- Authentication and authorization\n- TLS handling\n- Request parsing limits\n- Route reference\n- Response and error behavior" \
  "src/api/mod.rs" \
  "src/api/controller.rs" \
  "src/api/fallback.rs" \
  "src/api/worker.rs" \
  "tests/bdd_api_http.rs"

build_prompt \
  "debug-api" \
  "docs/src/reference/debug-api.md" \
  "Reference the debug snapshot machinery, verbose payload shape, UI endpoints, and debug worker publication loop." \
  "- Overview\n- Module surface\n- Snapshot model\n- Verbose payload sections\n- Published endpoints\n- Worker loop and history behavior" \
  "src/debug_api/mod.rs" \
  "src/debug_api/snapshot.rs" \
  "src/debug_api/view.rs" \
  "src/debug_api/worker.rs"

build_prompt \
  "ha-state-machine" \
  "docs/src/reference/ha-state-machine.md" \
  "Reference the HA phases, decision model, lowered effects, and worker state transitions." \
  "- Overview\n- Phase model\n- World snapshot and decision inputs\n- Decision variants\n- Effect and action lowering\n- Worker loop behavior\n- Selected invariants from tests when directly supported by the excerpts" \
  "src/ha/mod.rs" \
  "src/ha/state.rs" \
  "src/ha/decision.rs" \
  "src/ha/decide.rs" \
  "src/ha/lower.rs" \
  "src/ha/actions.rs" \
  "src/ha/apply.rs" \
  "src/ha/worker.rs"

build_prompt \
  "node-runtime" \
  "docs/src/reference/node-runtime.md" \
  "Reference node startup planning, startup execution phases, worker wiring, and runtime error surface." \
  "- Overview\n- Public entrypoints and binary contract\n- Startup planning types and outcomes\n- Startup execution flow\n- Worker startup and channel wiring\n- Runtime error variants and key constants" \
  "src/bin/pgtuskmaster.rs" \
  "src/runtime/mod.rs" \
  "src/runtime/node.rs" \
  "tests/cli_binary.rs"

build_prompt \
  "process-worker" \
  "docs/src/reference/process-worker.md" \
  "Reference process job types, worker state, command building, subprocess capture, and completion handling." \
  "- Overview\n- Module surface\n- Job and state types\n- Command runner surface\n- Worker loop and request handling\n- Active execution and output draining\n- Timeouts and preflight behavior" \
  "src/process/mod.rs" \
  "src/process/jobs.rs" \
  "src/process/state.rs" \
  "src/process/worker.rs"

build_prompt \
  "logging" \
  "docs/src/reference/logging.md" \
  "Reference the logging record model, sinks, raw record builders, file tailing, and PostgreSQL ingest worker." \
  "- Overview\n- Record model\n- Application event helpers\n- Builders and handles\n- Sink bootstrap and sink behaviors\n- File tailing\n- PostgreSQL ingest worker" \
  "src/logging/mod.rs" \
  "src/logging/event.rs" \
  "src/logging/raw_record.rs" \
  "src/logging/tailer.rs" \
  "src/logging/postgres_ingest.rs"

build_prompt \
  "managed-postgres" \
  "docs/src/reference/managed-postgres.md" \
  "Reference the managed PostgreSQL runtime-file materialization module and readback helpers." \
  "- Overview\n- Core types\n- Managed file set\n- Materialization pipeline\n- Standby auth materialization\n- TLS materialization\n- Signal-file behavior\n- Readback and runtime integration boundary" \
  "src/postgres_managed.rs" \
  "src/runtime/node.rs"

build_prompt \
  "managed-postgres-conf" \
  "docs/src/reference/managed-postgres-conf.md" \
  "Reference the managed PostgreSQL config model, primary_conninfo render and parse rules, and extra GUC validation surface." \
  "- Overview\n- Module constants\n- Core types\n- Rendered configuration model\n- Start-intent and recovery-signal mapping\n- Primary conninfo render and parse rules\n- Validation rules" \
  "src/postgres_managed_conf.rs"

build_prompt \
  "tls" \
  "docs/src/reference/tls.md" \
  "Reference the TLS config types, parser validation rules, rustls server-config assembly, runtime wiring, and API accept behavior." \
  "- Overview\n- Config types\n- Parser validation surface\n- Rustls builder behavior\n- Runtime wiring\n- API worker TLS behavior\n- Error variants and constants" \
  "src/tls.rs" \
  "src/config/parser.rs" \
  "src/runtime/node.rs" \
  "src/api/worker.rs"

build_prompt \
  "shared-state" \
  "docs/src/reference/shared-state.md" \
  "Reference the shared identifier wrappers, versioned snapshot types, worker status and error enums, and watch-channel publisher/subscriber API." \
  "- Overview\n- Module surface\n- Identifier and scalar wrapper types\n- Versioned snapshot and worker status types\n- Error enums\n- Watch-channel constructor and handles\n- Verified behaviors from tests when directly supported" \
  "src/state/mod.rs" \
  "src/state/errors.rs" \
  "src/state/ids.rs" \
  "src/state/time.rs" \
  "src/state/watch_state.rs" \
  "tests/bdd_state_watch.rs"

build_prompt \
  "pgtuskmaster" \
  "docs/src/reference/pgtuskmaster.md" \
  "Reference the pgtuskmaster node binary command surface, startup delegation flow, worker bootstrap, and exit behavior." \
  "- Overview\n- Binary command surface\n- Entry-point flow\n- Runtime delegation\n- Worker setup boundary\n- Exit behavior" \
  "src/bin/pgtuskmaster.rs" \
  "src/runtime/mod.rs" \
  "src/runtime/node.rs" \
  "tests/cli_binary.rs"

build_prompt \
  "pginfo" \
  "docs/src/reference/pginfo.md" \
  "Reference the pginfo module surface, conninfo parse/render helpers, poll query contract, published state model, and worker loop." \
  "- Overview\n- Module surface\n- Conninfo types and parsing\n- Poll query and decoded payload\n- Published state model\n- Worker loop and emitted events\n- Verified behaviors from direct tests" \
  "src/pginfo/mod.rs" \
  "src/pginfo/conninfo.rs" \
  "src/pginfo/query.rs" \
  "src/pginfo/state.rs" \
  "src/pginfo/worker.rs"

build_prompt \
  "pgtuskmasterctl" \
  "docs/src/reference/pgtuskmasterctl.md" \
  "Reference the pgtuskmasterctl CLI command tree, HTTP client mapping, output rendering, token selection, and exit-code behavior." \
  "- Overview\n- Binary entrypoint\n- Global options\n- Command tree\n- Client construction\n- HTTP mapping\n- Output rendering\n- Exit behavior" \
  "src/bin/pgtuskmasterctl.rs" \
  "src/cli/args.rs" \
  "src/cli/client.rs" \
  "src/cli/output.rs" \
  "src/cli/mod.rs" \
  "src/cli/error.rs" \
  "tests/cli_binary.rs"

printf '%s\n' \
  "$PROMPT_DIR/runtime-config.prompt.md" \
  "$PROMPT_DIR/dcs.prompt.md" \
  "$PROMPT_DIR/http-api.prompt.md" \
  "$PROMPT_DIR/debug-api.prompt.md" \
  "$PROMPT_DIR/ha-state-machine.prompt.md" \
  "$PROMPT_DIR/node-runtime.prompt.md" \
  "$PROMPT_DIR/process-worker.prompt.md" \
  "$PROMPT_DIR/logging.prompt.md" \
  "$PROMPT_DIR/managed-postgres.prompt.md" \
  "$PROMPT_DIR/managed-postgres-conf.prompt.md" \
  "$PROMPT_DIR/tls.prompt.md" \
  "$PROMPT_DIR/shared-state.prompt.md" \
  "$PROMPT_DIR/pgtuskmaster.prompt.md" \
  "$PROMPT_DIR/pginfo.prompt.md" \
  "$PROMPT_DIR/pgtuskmasterctl.prompt.md"
