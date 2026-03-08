You are drafting exactly one documentation file.

Rules:
- Follow Diataxis strictly.
- Use only the supplied repo facts and supplied Diataxis summary.
- If a fact is missing, say "missing source support" instead of inventing.
- ASCII only.
- No em dashes.
- Prefer diagrams only when the supplied facts support every node and edge.

You are being given a very large raw context pack on purpose.
Use it aggressively.
Do not assume something is unimportant just because it feels obvious.
Small implementation details, file names, command names, field names, and test evidence can materially improve the quality of the document.

You are writing for a repo where the supervising agent will do only minimal factual correction afterward.
That means you should make your own best content decisions from the supplied evidence.
Do not leave obvious useful material unused if it is present in the supplied files.
Do not ask for permission.
Do not defer decisions that can be made from the evidence.
Do not produce a meta-plan.
Produce the document itself.

Behavior requirements:
- Read the target path and infer the intended page boundary from it.
- Use the Diataxis type that best matches the supplied target and evidence.
- Let the supplied evidence determine scope, framing, and detail.
- Keep unsupported claims out of the document.
- If an important fact is missing, write "missing source support" at the exact point where that fact would otherwise be needed.
- Prefer exact terminology from the repo for commands, flags, config fields, and module names.
- Prefer exact field names when discussing config or API details.
- Prefer user-facing language when introducing a concept, but do not rename exact technical terms.
- Use the supplied existing docs content to avoid inconsistency with the current book.

Diagram rule:
- Include a diagram only if the supplied evidence supports every node and edge.
- If you include a diagram, keep it simple and factual.
- Mermaid is acceptable.

Content quality requirements:
- Write a real page, not notes.
- Use the supplied evidence to be concrete.
- Use examples when the evidence supports them.
- Do not hedge where the evidence is clear.
- Do not inflate the page with generic documentation filler.
- Do not add process commentary about how you wrote the page.
- Do not mention these instructions.

Task:
- Write the requested documentation file only.
- Stay within the supplied scope.
- Use the full raw context pack below, including full existing docs contents and full requested source files.
- If some fact is missing, write "missing source support" rather than inventing.


# target docs path

docs/src/introduction/single-node-tutorial.md

# docs/src file listing

# docs/src file listing

docs/src/SUMMARY.md


# full docs/src file contents


===== docs/src/SUMMARY.md =====
# Summary


# diataxis summary markdown

# Diataxis Framework Summary

## Tutorial: Learn Diataxis

### What You Will Accomplish
You will understand the four kinds of documentation and be able to classify any documentation page using the Diataxis compass.

### Steps

1. **Start with the core idea**
   - There are exactly four kinds of documentation: tutorials, how-to guides, reference, and explanation
   - These categories come from two dimensions of documentation need

2. **Understand the two dimensions**
   - **Action vs Cognition**: Does the content guide doing or provide knowledge?
   - **Acquisition vs Application**: Does it serve learning or work?

3. **Use the compass to classify**
   - Ask: "Is this action or cognition?"
   - Ask: "Is this acquisition or application?"
   - The intersection gives you the type

4. **Check your understanding**
   - Try classifying 3-5 pages from any documentation site
   - Use the compass table below as your reference

## How-to Guides

### How to Classify Documentation Using Diataxis

**Goal**: Determine the correct Diataxis category for any documentation content

**Prerequisites**: You must be looking at actual documentation content, not planning from scratch

**Steps**:

1. **Identify the content's primary purpose**
   - Read the first two paragraphs
   - Ask: "What is this trying to help me DO or KNOW?"

2. **Apply the compass test**
   - Question 1: Does the content inform ACTION (steps, doing) or COGNITION (facts, thinking)?
   - Question 2: Does it serve ACQUISITION (learning, study) or APPLICATION (work, tasks)?

3. **Determine the category**
   - Action + Acquisition = Tutorial
   - Action + Application = How-to guide
   - Cognition + Application = Reference
   - Cognition + Acquisition = Explanation

4. **Verify your classification**
   - Check if the language matches the category using the type checklist below
   - If substantial content belongs to another category, it may need splitting

### How to Structure Documentation Hierarchy

**Goal**: Organize documentation that serves multiple user types or deployment scenarios

**When to use**: When you have overlapping concerns such as different user groups or deployment environments

**Steps**:

1. **Identify user segments**
   - List the distinct user groups or contexts
   - Verify they have meaningfully different documentation needs

2. **Choose primary dimension**
   - Option A: Diataxis categories at top level, user segments beneath
   - Option B: User segments at top level, Diataxis categories beneath

3. **Evaluate shared content**
   - If content is mostly shared, prefer one arrangement
   - If content is mostly distinct, prefer the other

4. **Create landing pages when the content volume justifies them**
   - For each section, write overview text rather than only lists
   - Group related items into smaller subsections when lists grow too long

## Reference

### The Four Documentation Types

| Type | Purpose | Serves | Content Focus | Answers |
|------|---------|--------|---------------|---------|
| Tutorial | Learning experience | Acquisition of skill | Practical steps under guidance | "Can you teach me to...?" |
| How-to guide | Practical directions | Application of skill | Actions to solve a problem | "How do I...?" |
| Reference | Technical description | Application of skill | Facts about the machinery | "What is...?" |
| Explanation | Discursive treatment | Acquisition of skill | Understanding and context | "Why...?" |

### Compass Decision Table

| If the content... | ...and serves the user's... | ...then it must belong to... |
|-------------------|-----------------------------|------------------------------|
| informs action | acquisition of skill | a tutorial |
| informs action | application of skill | a how-to guide |
| informs cognition | application of skill | reference |
| informs cognition | acquisition of skill | explanation |

### Terminology Mapping

- **Action** = practical steps, doing
- **Cognition** = theoretical knowledge, thinking
- **Acquisition** = study, learning
- **Application** = work, tasks

## Explanation

### Why the Four Types Are Sufficient

The Diataxis framework identifies exactly four documentation types because it maps to the complete territory of human skill development. Two dimensions define documentation needs:

1. **Action/Cognition**: Documentation either guides action or informs cognition

2. **Acquisition/Application**: The user is either acquiring skill or applying skill

These dimensions create four quarters. In the Diataxis model, these quarters define the complete territory of craft documentation.

### When Intuition Fails

The map is reliable but intuition can mislead. Common failure patterns:

- **Tutorial/How-to conflation**: Tutorials teach; how-to guides direct work
- **Reference/Explanation blending**: Explanation creeping into reference material obscures facts
- **Partial collapse**: When boundaries blur, documentation becomes less effective

The compass tool prevents these errors by forcing explicit classification through the two key questions.

### User Cycle Interaction

Users move through documentation types cyclically, but not necessarily in order:

- **Learning phase**: Tutorial
- **Goal phase**: How-to guide
- **Information phase**: Reference
- **Understanding phase**: Explanation

Then the cycle repeats at deeper levels or for new skills.

### Quality Dimensions

**Functional quality**:
- Accuracy, completeness, consistency, precision
- Independent characteristics, objectively measurable

**Deep quality**:
- Feels good to use, has flow, fits human needs
- Interdependent characteristics, subjectively assessed
- Depends on functional quality

Diataxis helps expose functional quality gaps and supports deep quality by structuring documentation around user needs.

---

## LLM Drafting Checklist

### Before Writing Any Page

- [ ] Identify which of the four types this page will be
- [ ] Run the compass test: action/cognition? acquisition/application?
- [ ] Check that the page is dominated by a single type
- [ ] Write the page title to clearly signal the type

### Tutorial Checklist

- [ ] Uses "We will..." not "You will learn..."
- [ ] Starts with concrete, particular tools/materials
- [ ] Provides expected output after every step
- [ ] Contains minimal explanation
- [ ] Has no choices, alternatives, or branches
- [ ] Ends with a meaningful, visible result
- [ ] Could be repeated by a learner for practice

### How-to Guide Checklist

- [ ] Title starts with "How to..." or equivalent
- [ ] Addresses a specific, real-world problem
- [ ] Assumes user already knows what they want to achieve
- [ ] Contains only actions and conditional logic where needed
- [ ] No explanatory digressions, link out if needed
- [ ] Starts and ends at reasonable, meaningful points
- [ ] Prepares for unexpected situations when relevant

### Reference Checklist

- [ ] Structure mirrors the product/code structure
- [ ] Contains only facts, descriptions, and specifications
- [ ] Uses neutral, objective language throughout
- [ ] No opinions, explanations, or instructions
- [ ] Provides examples only for illustration
- [ ] Follows consistent patterns and formats
- [ ] Can be consulted, not read linearly

### Explanation Checklist

- [ ] Title could be prefixed with "About..."
- [ ] Makes connections between concepts
- [ ] Provides context: history, design decisions, constraints
- [ ] Discusses alternatives, trade-offs, perspectives
- [ ] Contains deliberate perspective when relevant
- [ ] Stays within bounded topic
- [ ] Written for reflection away from the product

### Classification Emergency Protocol

If you cannot classify after 30 seconds:
1. Answer: Is this about DOING or KNOWING?
2. Answer: Is this for LEARNING or WORKING?
3. Look up the intersection in the compass table
4. If still unclear, the content is probably mixed and needs splitting


# project manifests and docs config

===== Cargo.toml =====
[package]
name = "pgtuskmaster_rust"
version = "0.1.0"
edition = "2021"

[features]
default = []

[dependencies]
clap = { version = "4.5.47", features = ["derive", "env"] }
serde = { version = "1.0.219", features = ["derive"] }
serde_json = "1.0.140"
sha2 = "0.10.9"
thiserror = "2.0.12"
tokio = { version = "1.44.1", features = ["sync", "rt", "rt-multi-thread", "macros", "time", "process", "net", "io-util", "fs"] }
tokio-postgres = "0.7.13"
toml = "0.8.20"
httparse = "1.10.1"
etcd-client = "0.14.1"
reqwest = { version = "0.12.24", default-features = false, features = ["blocking", "json", "rustls-tls"] }
rustls = { version = "0.23.28", features = ["ring"] }
rustls-pemfile = "2.2.0"
tokio-rustls = "0.26.4"
tracing = "0.1.41"
tracing-subscriber = "0.3.20"

[target.'cfg(unix)'.dependencies]
libc = "0.2"

[dev-dependencies]
rcgen = "0.14.5"


===== docs/book.toml =====
[book]
authors = ["Joshua Azimullah"]
language = "en"
multilingual = false
src = "src"
title = "pgtuskmaster"

[preprocessor.mermaid]
command = "mdbook-mermaid"

[output]

[output.html]
additional-js = ["mermaid.min.js", "mermaid-init.js"]




# src and test file listing

# src and test file listing

src/api/controller.rs
src/api/fallback.rs
src/api/mod.rs
src/api/worker.rs
src/bin/pgtuskmaster.rs
src/bin/pgtuskmasterctl.rs
src/cli/args.rs
src/cli/client.rs
src/cli/error.rs
src/cli/mod.rs
src/cli/output.rs
src/config/defaults.rs
src/config/mod.rs
src/config/parser.rs
src/config/schema.rs
src/dcs/etcd_store.rs
src/dcs/keys.rs
src/dcs/mod.rs
src/dcs/state.rs
src/dcs/store.rs
src/dcs/worker.rs
src/debug_api/mod.rs
src/debug_api/snapshot.rs
src/debug_api/view.rs
src/debug_api/worker.rs
src/ha/actions.rs
src/ha/apply.rs
src/ha/decide.rs
src/ha/decision.rs
src/ha/events.rs
src/ha/lower.rs
src/ha/mod.rs
src/ha/process_dispatch.rs
src/ha/source_conn.rs
src/ha/state.rs
src/ha/worker.rs
src/lib.rs
src/logging/event.rs
src/logging/mod.rs
src/logging/postgres_ingest.rs
src/logging/raw_record.rs
src/logging/tailer.rs
src/pginfo/conninfo.rs
src/pginfo/mod.rs
src/pginfo/query.rs
src/pginfo/state.rs
src/pginfo/worker.rs
src/postgres_managed.rs
src/postgres_managed_conf.rs
src/process/jobs.rs
src/process/mod.rs
src/process/state.rs
src/process/worker.rs
src/runtime/mod.rs
src/runtime/node.rs
src/state/errors.rs
src/state/ids.rs
src/state/mod.rs
src/state/time.rs
src/state/watch_state.rs
src/test_harness/auth.rs
src/test_harness/binaries.rs
src/test_harness/etcd3.rs
src/test_harness/ha_e2e/config.rs
src/test_harness/ha_e2e/handle.rs
src/test_harness/ha_e2e/mod.rs
src/test_harness/ha_e2e/ops.rs
src/test_harness/ha_e2e/startup.rs
src/test_harness/ha_e2e/util.rs
src/test_harness/mod.rs
src/test_harness/namespace.rs
src/test_harness/net_proxy.rs
src/test_harness/pg16.rs
src/test_harness/ports.rs
src/test_harness/provenance.rs
src/test_harness/runtime_config.rs
src/test_harness/signals.rs
src/test_harness/tls.rs
src/tls.rs
src/worker_contract_tests.rs
tests/bdd_api_http.rs
tests/bdd_state_watch.rs
tests/cli_binary.rs
tests/ha/support/multi_node.rs
tests/ha/support/observer.rs
tests/ha/support/partition.rs
tests/ha_multi_node_failover.rs
tests/ha_partition_isolation.rs
tests/policy_e2e_api_only.rs


# docker and docs support file listing

docker/Dockerfile.dev
docker/Dockerfile.prod
docker/compose/docker-compose.cluster.yml
docker/compose/docker-compose.single.yml
docker/configs/cluster/node-a/runtime.toml
docker/configs/cluster/node-b/runtime.toml
docker/configs/cluster/node-c/runtime.toml
docker/configs/common/pg_hba.conf
docker/configs/common/pg_ident.conf
docker/configs/single/node-a/runtime.toml
docker/entrypoint.sh
docker/secrets/postgres-superuser.password.example
docker/secrets/replicator.password.example
docker/secrets/rewinder.password.example
docs/book.toml
docs/draft/docs/src/introduction/overview.md
docs/draft/docs/src/introduction/single-node-tutorial.md
docs/mermaid-init.js
docs/mermaid.min.js
docs/src/SUMMARY.md
docs/tmp/docs/src/introduction/overview.prompt.md
docs/tmp/docs/src/introduction/single-node-tutorial.prompt.md


===== docker/compose/docker-compose.single.yml =====
services:
  etcd:
    image: ${ETCD_IMAGE}
    command:
      - etcd
      - --name=etcd
      - --data-dir=/etcd-data
      - --listen-client-urls=http://0.0.0.0:2379
      - --advertise-client-urls=http://etcd:2379
      - --listen-peer-urls=http://0.0.0.0:2380
      - --initial-advertise-peer-urls=http://etcd:2380
      - --initial-cluster=etcd=http://etcd:2380
      - --initial-cluster-state=new
    healthcheck:
      test: ["CMD", "etcdctl", "--endpoints=http://127.0.0.1:2379", "endpoint", "health"]
      interval: 5s
      timeout: 5s
      retries: 20
    networks:
      - pgtm-internal
    volumes:
      - etcd-single-data:/etcd-data

  node-a:
    image: ${PGTUSKMASTER_IMAGE}
    build:
      context: ../..
      dockerfile: docker/Dockerfile.prod
    depends_on:
      etcd:
        condition: service_healthy
    restart: unless-stopped
    configs:
      - source: runtime-node-a
        target: /etc/pgtuskmaster/runtime.toml
      - source: common-pg-hba
        target: /etc/pgtuskmaster/pg_hba.conf
      - source: common-pg-ident
        target: /etc/pgtuskmaster/pg_ident.conf
    secrets:
      - source: superuser-password
        target: postgres-superuser-password
      - source: replicator-password
        target: replicator-password
      - source: rewinder-password
        target: rewinder-password
    networks:
      - pgtm-internal
    ports:
      - "${PGTM_SINGLE_API_PORT}:8080"
      - "${PGTM_SINGLE_PG_PORT}:5432"
    volumes:
      - node-a-single-data:/var/lib/postgresql
      - node-a-single-logs:/var/log/pgtuskmaster

configs:
  runtime-node-a:
    file: ../configs/single/node-a/runtime.toml
  common-pg-hba:
    file: ../configs/common/pg_hba.conf
  common-pg-ident:
    file: ../configs/common/pg_ident.conf

secrets:
  superuser-password:
    file: ${PGTM_SECRET_SUPERUSER_FILE}
  replicator-password:
    file: ${PGTM_SECRET_REPLICATOR_FILE}
  rewinder-password:
    file: ${PGTM_SECRET_REWINDER_FILE}

networks:
  pgtm-internal:
    driver: bridge

volumes:
  etcd-single-data:
  node-a-single-data:
  node-a-single-logs:


===== docker/configs/single/node-a/runtime.toml =====
config_version = "v2"

[cluster]
name = "docker-single"
member_id = "node-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "node-a"
listen_port = 5432
socket_dir = "/var/lib/pgtuskmaster/socket"
log_file = "/var/log/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
rewind_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
tls = { mode = "disabled" }
pg_hba = { source = { path = "/etc/pgtuskmaster/pg_hba.conf" } }
pg_ident = { source = { path = "/etc/pgtuskmaster/pg_ident.conf" } }

[postgres.roles.superuser]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/postgres-superuser-password" } }

[postgres.roles.replicator]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/replicator-password" } }

[postgres.roles.rewinder]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/rewinder-password" } }

[dcs]
endpoints = ["http://etcd:2379"]
scope = "docker-single"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000

[process.binaries]
postgres = "/usr/lib/postgresql/16/bin/postgres"
pg_ctl = "/usr/lib/postgresql/16/bin/pg_ctl"
pg_rewind = "/usr/lib/postgresql/16/bin/pg_rewind"
initdb = "/usr/lib/postgresql/16/bin/initdb"
pg_basebackup = "/usr/lib/postgresql/16/bin/pg_basebackup"
psql = "/usr/lib/postgresql/16/bin/psql"

[logging]
level = "info"
capture_subprocess_output = true

[logging.postgres]
enabled = true
poll_interval_ms = 200
cleanup = { enabled = true, max_files = 20, max_age_seconds = 86400, protect_recent_seconds = 300 }

[logging.sinks.stderr]
enabled = true

[logging.sinks.file]
enabled = true
path = "/var/log/pgtuskmaster/runtime.jsonl"
mode = "append"

[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }

[debug]
enabled = true


===== docker/configs/common/pg_hba.conf =====
local   all             all                                     trust
host    all             all             127.0.0.1/32            trust
host    all             all             ::1/128                 trust
host    all             all             0.0.0.0/0               trust
host    replication     all             127.0.0.1/32            trust
host    replication     all             0.0.0.0/0               trust


===== docker/secrets/postgres-superuser.password.example =====
replace-with-a-strong-superuser-password


===== docker/secrets/replicator.password.example =====
replace-with-a-strong-replicator-password


===== docker/secrets/rewinder.password.example =====
replace-with-a-strong-rewinder-password


===== docker/entrypoint.sh =====
#!/usr/bin/env bash

set -euo pipefail

readonly DEFAULT_CONFIG_PATH="/etc/pgtuskmaster/runtime.toml"

die() {
    printf 'pgtuskmaster entrypoint error: %s\n' "$*" >&2
    exit 1
}

require_readable_file() {
    local label="$1"
    local path="$2"

    [[ -e "${path}" ]] || die "${label} does not exist: ${path}"
    [[ -f "${path}" ]] || die "${label} is not a regular file: ${path}"
    [[ -r "${path}" ]] || die "${label} is not readable: ${path}"
}

require_executable_file() {
    local label="$1"
    local path="$2"

    [[ -e "${path}" ]] || die "${label} does not exist: ${path}"
    [[ -f "${path}" ]] || die "${label} is not a regular file: ${path}"
    [[ -x "${path}" ]] || die "${label} is not executable: ${path}"
}

ensure_directory() {
    local label="$1"
    local path="$2"

    if [[ -e "${path}" ]]; then
        [[ -d "${path}" ]] || die "${label} exists but is not a directory: ${path}"
        [[ -w "${path}" ]] || die "${label} is not writable: ${path}"
        return
    fi

    mkdir -p "${path}" || die "failed to create ${label}: ${path}"
}

ensure_parent_directory() {
    local label="$1"
    local path="$2"
    local parent

    parent="$(dirname "${path}")"
    ensure_directory "${label} parent directory" "${parent}"
}

extract_first_key_path() {
    local key="$1"
    local config_path="$2"

    awk -v key="${key}" '
        $0 ~ "^[[:space:]]*" key "[[:space:]]*=[[:space:]]*\"" {
            line = $0
            sub(/^[^"]*"/, "", line)
            sub(/".*$/, "", line)
            print line
            exit
        }
    ' "${config_path}"
}

emit_section_paths() {
    local config_path="$1"

    awk '
        /^[[:space:]]*\[/ {
            section = $0
            gsub(/^[[:space:]]*\[/, "", section)
            gsub(/\][[:space:]]*$/, "", section)
            next
        }
        /path[[:space:]]*=[[:space:]]*"/ {
            path = $0
            sub(/^.*path[[:space:]]*=[[:space:]]*"/, "", path)
            sub(/".*$/, "", path)
            printf "%s\t%s\n", section, path
        }
    ' "${config_path}"
}

validate_runtime_contract() {
    local config_path="$1"
    local binary_name
    local binary_path
    local directory_key
    local directory_path
    local file_key
    local file_path
    local section_path
    local section
    local path

    for binary_name in postgres pg_ctl pg_rewind initdb pg_basebackup psql; do
        binary_path="$(extract_first_key_path "${binary_name}" "${config_path}")"
        [[ -n "${binary_path}" ]] || die "missing process.binaries.${binary_name} in ${config_path}"
        require_executable_file "process.binaries.${binary_name}" "${binary_path}"
    done

    for directory_key in data_dir socket_dir log_dir; do
        directory_path="$(extract_first_key_path "${directory_key}" "${config_path}")"
        if [[ -n "${directory_path}" ]]; then
            ensure_directory "${directory_key}" "${directory_path}"
        fi
    done

    for file_key in log_file pg_ctl_log_file; do
        file_path="$(extract_first_key_path "${file_key}" "${config_path}")"
        if [[ -n "${file_path}" ]]; then
            ensure_parent_directory "${file_key}" "${file_path}"
        fi
    done

    while IFS=$'\t' read -r section path; do
        [[ -n "${path}" ]] || continue

        case "${section}" in
            postgres.pg_hba|postgres.pg_ident|postgres.roles.superuser.auth.password|postgres.roles.replicator.auth.password|postgres.roles.rewinder.auth.password|postgres.tls.identity|postgres.tls.client_auth|api.security.tls.identity|api.security.tls.client_auth)
                require_readable_file "${section}.path" "${path}"
                ;;
            logging.sinks.file)
                ensure_parent_directory "${section}.path" "${path}"
                ;;
            *)
                if [[ "${path}" == /run/secrets/* ]]; then
                    require_readable_file "${section}.path" "${path}"
                fi
                ;;
        esac
    done < <(emit_section_paths "${config_path}")

    while IFS= read -r section_path; do
        [[ -n "${section_path}" ]] || continue
        require_readable_file "referenced docker secret" "${section_path}"
    done < <(
        grep -oE '/run/secrets/[^"[:space:],}]+' "${config_path}" | sort -u
    )
}

main() {
    local config_path="${PGTUSKMASTER_CONFIG:-${DEFAULT_CONFIG_PATH}}"

    umask 077

    if [[ "$#" -eq 0 ]]; then
        set -- /usr/local/bin/pgtuskmaster --config "${config_path}"
    elif [[ "$1" == "pgtuskmaster" || "$1" == "/usr/local/bin/pgtuskmaster" ]]; then
        set -- /usr/local/bin/pgtuskmaster --config "${config_path}" "${@:2}"
    else
        exec "$@"
    fi

    require_readable_file "runtime config" "${config_path}"
    validate_runtime_contract "${config_path}"

    exec "$@"
}

main "$@"


===== src/bin/pgtuskmaster.rs =====
use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "pgtuskmaster")]
#[command(about = "Run a pgtuskmaster node")]
struct Cli {
    /// Path to runtime config file
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    run_node(cli)
}

fn run_node(cli: Cli) -> ExitCode {
    let config = match cli.config.as_ref() {
        Some(path) => path,
        None => {
            eprintln!("missing required `--config <PATH>`");
            return ExitCode::from(2);
        }
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .map_err(|err| err.to_string());
    let runtime = match runtime {
        Ok(value) => value,
        Err(err) => {
            eprintln!("failed to build tokio runtime: {err}");
            return ExitCode::from(1);
        }
    };

    let result = runtime.block_on(pgtuskmaster_rust::runtime::run_node_from_config_path(
        config.as_path(),
    ));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}


===== src/config/schema.rs =====
use std::{collections::BTreeMap, fmt, path::PathBuf};

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigVersion {
    V1,
    V2,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum InlineOrPath {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct SecretSource(pub InlineOrPath);

impl fmt::Debug for SecretSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            InlineOrPath::Path(path) => f
                .debug_tuple("SecretSource")
                .field(&format_args!("path({})", path.display()))
                .finish(),
            InlineOrPath::PathConfig { path } => f
                .debug_tuple("SecretSource")
                .field(&format_args!("path({})", path.display()))
                .finish(),
            InlineOrPath::Inline { .. } => f
                .debug_tuple("SecretSource")
                .field(&"<inline redacted>")
                .finish(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiTlsMode {
    Disabled,
    Optional,
    Required,
}

pub type TlsMode = ApiTlsMode;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerIdentityConfig {
    pub cert_chain: InlineOrPath,
    pub private_key: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsClientAuthConfig {
    pub client_ca: InlineOrPath,
    pub require_client_cert: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerConfig {
    pub mode: TlsMode,
    pub identity: Option<TlsServerIdentityConfig>,
    pub client_auth: Option<TlsClientAuthConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RuntimeConfig {
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfig,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: ProcessConfig,
    pub logging: LoggingConfig,
    pub api: ApiConfig,
    pub debug: DebugConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterConfig {
    pub name: String,
    pub member_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfig {
    pub data_dir: PathBuf,
    pub connect_timeout_s: u32,
    pub listen_host: String,
    pub listen_port: u16,
    pub socket_dir: PathBuf,
    pub log_file: PathBuf,
    pub local_conn_identity: PostgresConnIdentityConfig,
    pub rewind_conn_identity: PostgresConnIdentityConfig,
    pub tls: TlsServerConfig,
    pub roles: PostgresRolesConfig,
    pub pg_hba: PgHbaConfig,
    pub pg_ident: PgIdentConfig,
    pub extra_gucs: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConnIdentityConfig {
    pub user: String,
    pub dbname: String,
    pub ssl_mode: crate::pginfo::conninfo::PgSslMode,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RoleAuthConfig {
    Tls,
    Password { password: SecretSource },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRoleConfig {
    pub username: String,
    pub auth: RoleAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRolesConfig {
    pub superuser: PostgresRoleConfig,
    pub replicator: PostgresRoleConfig,
    pub rewinder: PostgresRoleConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgHbaConfig {
    pub source: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgIdentConfig {
    pub source: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsConfig {
    pub endpoints: Vec<String>,
    pub scope: String,
    pub init: Option<DcsInitConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsInitConfig {
    pub payload_json: String,
    pub write_on_bootstrap: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaConfig {
    pub loop_interval_ms: u64,
    pub lease_ttl_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfig {
    pub pg_rewind_timeout_ms: u64,
    pub bootstrap_timeout_ms: u64,
    pub fencing_timeout_ms: u64,
    pub binaries: BinaryPaths,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub capture_subprocess_output: bool,
    pub postgres: PostgresLoggingConfig,
    pub sinks: LoggingSinksConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresLoggingConfig {
    pub enabled: bool,
    pub pg_ctl_log_file: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    pub poll_interval_ms: u64,
    pub cleanup: LogCleanupConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingSinksConfig {
    pub stderr: StderrSinkConfig,
    pub file: FileSinkConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StderrSinkConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSinkConfig {
    pub enabled: bool,
    pub path: Option<PathBuf>,
    pub mode: FileSinkMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSinkMode {
    Append,
    Truncate,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogCleanupConfig {
    pub enabled: bool,
    pub max_files: u64,
    pub max_age_seconds: u64,
    #[serde(default = "default_log_cleanup_protect_recent_seconds")]
    pub protect_recent_seconds: u64,
}

fn default_log_cleanup_protect_recent_seconds() -> u64 {
    300
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryPaths {
    pub postgres: PathBuf,
    pub pg_ctl: PathBuf,
    pub pg_rewind: PathBuf,
    pub initdb: PathBuf,
    pub pg_basebackup: PathBuf,
    pub psql: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfig {
    pub listen_addr: String,
    pub security: ApiSecurityConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiSecurityConfig {
    pub tls: TlsServerConfig,
    pub auth: ApiAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiAuthConfig {
    Disabled,
    RoleTokens(ApiRoleTokensConfig),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiRoleTokensConfig {
    pub read_token: Option<String>,
    pub admin_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialRuntimeConfig {
    pub cluster: ClusterConfig,
    pub postgres: PartialPostgresConfig,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: PartialProcessConfig,
    pub logging: Option<PartialLoggingConfig>,
    pub api: Option<PartialApiConfig>,
    pub debug: Option<PartialDebugConfig>,
    pub security: Option<PartialSecurityConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialPostgresConfig {
    pub data_dir: PathBuf,
    pub connect_timeout_s: Option<u32>,
    pub listen_host: Option<String>,
    pub listen_port: Option<u16>,
    pub socket_dir: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialProcessConfig {
    pub pg_rewind_timeout_ms: Option<u64>,
    pub bootstrap_timeout_ms: Option<u64>,
    pub fencing_timeout_ms: Option<u64>,
    pub binaries: BinaryPaths,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialLoggingConfig {
    pub level: Option<LogLevel>,
    pub capture_subprocess_output: Option<bool>,
    pub postgres: Option<PartialPostgresLoggingConfig>,
    pub sinks: Option<PartialLoggingSinksConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialPostgresLoggingConfig {
    pub enabled: Option<bool>,
    pub pg_ctl_log_file: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    pub poll_interval_ms: Option<u64>,
    pub cleanup: Option<PartialLogCleanupConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialLogCleanupConfig {
    pub enabled: Option<bool>,
    pub max_files: Option<u64>,
    pub max_age_seconds: Option<u64>,
    pub protect_recent_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialLoggingSinksConfig {
    pub stderr: Option<PartialStderrSinkConfig>,
    pub file: Option<PartialFileSinkConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialStderrSinkConfig {
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialFileSinkConfig {
    pub enabled: Option<bool>,
    pub path: Option<PathBuf>,
    pub mode: Option<FileSinkMode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialApiConfig {
    pub listen_addr: Option<String>,
    pub read_auth_token: Option<String>,
    pub admin_auth_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialDebugConfig {
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialSecurityConfig {
    pub tls_enabled: Option<bool>,
    pub auth_token: Option<String>,
}

// -------------------------------
// v2 input schema (explicit secure)
// -------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfigV2Input {
    pub config_version: ConfigVersion,
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfigV2Input,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: ProcessConfigV2Input,
    pub logging: Option<LoggingConfig>,
    pub api: ApiConfigV2Input,
    pub debug: Option<DebugConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfigV2Input {
    pub pg_rewind_timeout_ms: Option<u64>,
    pub bootstrap_timeout_ms: Option<u64>,
    pub fencing_timeout_ms: Option<u64>,
    pub binaries: Option<BinaryPathsV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryPathsV2Input {
    pub postgres: Option<PathBuf>,
    pub pg_ctl: Option<PathBuf>,
    pub pg_rewind: Option<PathBuf>,
    pub initdb: Option<PathBuf>,
    pub pg_basebackup: Option<PathBuf>,
    pub psql: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfigV2Input {
    pub listen_addr: Option<String>,
    pub security: Option<ApiSecurityConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiSecurityConfigV2Input {
    pub tls: Option<TlsServerConfigV2Input>,
    pub auth: Option<ApiAuthConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfigV2Input {
    pub data_dir: PathBuf,
    pub connect_timeout_s: Option<u32>,
    pub listen_host: String,
    pub listen_port: u16,
    pub socket_dir: PathBuf,
    pub log_file: PathBuf,
    pub local_conn_identity: Option<PostgresConnIdentityConfigV2Input>,
    pub rewind_conn_identity: Option<PostgresConnIdentityConfigV2Input>,
    pub tls: Option<TlsServerConfigV2Input>,
    pub roles: Option<PostgresRolesConfigV2Input>,
    pub pg_hba: Option<PgHbaConfigV2Input>,
    pub pg_ident: Option<PgIdentConfigV2Input>,
    pub extra_gucs: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConnIdentityConfigV2Input {
    pub user: Option<String>,
    pub dbname: Option<String>,
    pub ssl_mode: Option<crate::pginfo::conninfo::PgSslMode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRoleConfigV2Input {
    pub username: Option<String>,
    pub auth: Option<RoleAuthConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRolesConfigV2Input {
    pub superuser: Option<PostgresRoleConfigV2Input>,
    pub replicator: Option<PostgresRoleConfigV2Input>,
    pub rewinder: Option<PostgresRoleConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgHbaConfigV2Input {
    pub source: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgIdentConfigV2Input {
    pub source: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerIdentityConfigV2Input {
    pub cert_chain: Option<InlineOrPath>,
    pub private_key: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RoleAuthConfigV2Input {
    Tls,
    Password { password: Option<SecretSource> },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerConfigV2Input {
    pub mode: Option<TlsMode>,
    pub identity: Option<TlsServerIdentityConfigV2Input>,
    pub client_auth: Option<TlsClientAuthConfig>,
}
