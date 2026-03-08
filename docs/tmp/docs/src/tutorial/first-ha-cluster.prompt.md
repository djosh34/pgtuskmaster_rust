You are drafting exactly one documentation file.

Rules:
- Follow Diataxis strictly.
- Use only the supplied repo facts and supplied Diataxis summary.
- If a fact is missing, say "missing source support" instead of inventing.
- ASCII only.
- No em dashes.
- Prefer diagrams only when the supplied facts support every node and edge.

Behavior requirements:
- Read the target path and infer the intended page boundary from it.
- Use the Diataxis type that best matches the supplied target and evidence.
- Keep unsupported claims out of the document.
- If an important fact is missing, write "missing source support" at the exact point where that fact would otherwise be needed.

Follow Diataxis method, write one real page, and include diagrams when needed using the syntax:

[diagram about x, y showing relation between z and a, **more details on diagram**]


# target docs path

docs/src/tutorial/first-ha-cluster.md

# docs/src file listing

# docs/src file listing

docs/src/SUMMARY.md


# current docs summary context

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
docs/draft/docs/src/tutorial/first-ha-cluster.md
docs/mermaid-init.js
docs/mermaid.min.js
docs/src/SUMMARY.md
docs/tmp/docs/src/tutorial/first-ha-cluster.prompt.md
docs/tmp/verbose_extra_context/cluster-start-command.md
docs/tmp/verbose_extra_context/leader-check-command.md


===== docker/compose/docker-compose.cluster.yml =====
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
      - etcd-cluster-data:/etcd-data

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
      - "${PGTM_CLUSTER_NODE_A_API_PORT}:8080"
      - "${PGTM_CLUSTER_NODE_A_PG_PORT}:5432"
    volumes:
      - node-a-cluster-data:/var/lib/postgresql
      - node-a-cluster-logs:/var/log/pgtuskmaster

  node-b:
    image: ${PGTUSKMASTER_IMAGE}
    build:
      context: ../..
      dockerfile: docker/Dockerfile.prod
    depends_on:
      etcd:
        condition: service_healthy
    restart: unless-stopped
    configs:
      - source: runtime-node-b
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
      - "${PGTM_CLUSTER_NODE_B_API_PORT}:8080"
      - "${PGTM_CLUSTER_NODE_B_PG_PORT}:5432"
    volumes:
      - node-b-cluster-data:/var/lib/postgresql
      - node-b-cluster-logs:/var/log/pgtuskmaster

  node-c:
    image: ${PGTUSKMASTER_IMAGE}
    build:
      context: ../..
      dockerfile: docker/Dockerfile.prod
    depends_on:
      etcd:
        condition: service_healthy
    restart: unless-stopped
    configs:
      - source: runtime-node-c
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
      - "${PGTM_CLUSTER_NODE_C_API_PORT}:8080"
      - "${PGTM_CLUSTER_NODE_C_PG_PORT}:5432"
    volumes:
      - node-c-cluster-data:/var/lib/postgresql
      - node-c-cluster-logs:/var/log/pgtuskmaster

configs:
  runtime-node-a:
    file: ../configs/cluster/node-a/runtime.toml
  runtime-node-b:
    file: ../configs/cluster/node-b/runtime.toml
  runtime-node-c:
    file: ../configs/cluster/node-c/runtime.toml
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
  etcd-cluster-data:
  node-a-cluster-data:
  node-a-cluster-logs:
  node-b-cluster-data:
  node-b-cluster-logs:
  node-c-cluster-data:
  node-c-cluster-logs:


===== docker/configs/cluster/node-a/runtime.toml =====
config_version = "v2"

[cluster]
name = "docker-cluster"
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
scope = "docker-cluster"

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


===== docker/configs/cluster/node-b/runtime.toml =====
config_version = "v2"

[cluster]
name = "docker-cluster"
member_id = "node-b"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "node-b"
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
scope = "docker-cluster"

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


===== docker/configs/cluster/node-c/runtime.toml =====
config_version = "v2"

[cluster]
name = "docker-cluster"
member_id = "node-c"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "node-c"
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
scope = "docker-cluster"

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


===== src/cli/args.rs =====
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
}

#[derive(Clone, Debug, Parser)]
#[command(name = "pgtuskmasterctl")]
#[command(about = "HA admin CLI for PGTuskMaster API")]
pub struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    pub base_url: String,
    #[arg(long, env = "PGTUSKMASTER_READ_TOKEN")]
    pub read_token: Option<String>,
    #[arg(long, env = "PGTUSKMASTER_ADMIN_TOKEN")]
    pub admin_token: Option<String>,
    #[arg(long, default_value_t = 5_000)]
    pub timeout_ms: u64,
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub output: OutputFormat,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    Ha(HaArgs),
}

#[derive(Clone, Debug, Args)]
pub struct HaArgs {
    #[command(subcommand)]
    pub command: HaCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum HaCommand {
    State,
    Switchover(SwitchoverArgs),
}

#[derive(Clone, Debug, Args)]
pub struct SwitchoverArgs {
    #[command(subcommand)]
    pub command: SwitchoverCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SwitchoverCommand {
    Clear,
    Request(RequestSwitchoverArgs),
}

#[derive(Clone, Debug, Args)]
pub struct RequestSwitchoverArgs {
    #[arg(long)]
    pub requested_by: String,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::args::{Cli, Command, HaCommand, OutputFormat, SwitchoverCommand};

    #[test]
    fn parse_ha_state_with_defaults() -> Result<(), String> {
        let cli = Cli::try_parse_from(["pgtuskmasterctl", "ha", "state"])
            .map_err(|err| format!("parse should succeed: {err}"))?;

        assert_eq!(cli.base_url, "http://127.0.0.1:8080");
        assert_eq!(cli.timeout_ms, 5_000);
        assert_eq!(cli.output, OutputFormat::Json);

        match cli.command {
            Command::Ha(ha) => match ha.command {
                HaCommand::State => Ok(()),
                _ => Err("expected ha state command".to_string()),
            },
        }
    }

    #[test]
    fn parse_requires_requested_by_for_switchover_request() {
        let parsed = Cli::try_parse_from(["pgtuskmasterctl", "ha", "switchover", "request"]);
        assert!(parsed.is_err(), "requested-by is required");
    }

    #[test]
    fn parse_full_switchover_write_command() -> Result<(), String> {
        let cli = Cli::try_parse_from([
            "pgtuskmasterctl",
            "--base-url",
            "https://cluster.example",
            "--timeout-ms",
            "1234",
            "--output",
            "text",
            "ha",
            "switchover",
            "request",
            "--requested-by",
            "node-a",
        ])
        .map_err(|err| format!("parse should succeed: {err}"))?;

        assert_eq!(cli.base_url, "https://cluster.example");
        assert_eq!(cli.timeout_ms, 1234);
        assert_eq!(cli.output, OutputFormat::Text);

        match cli.command {
            Command::Ha(ha) => match ha.command {
                HaCommand::Switchover(switchover) => match switchover.command {
                    SwitchoverCommand::Request(request) => {
                        assert_eq!(request.requested_by, "node-a");
                        Ok(())
                    }
                    _ => Err("expected switchover request".to_string()),
                },
                _ => Err("expected switchover command".to_string()),
            },
        }
    }

    #[test]
    fn parse_switchover_request() -> Result<(), String> {
        let cli = Cli::try_parse_from([
            "pgtuskmasterctl",
            "ha",
            "switchover",
            "request",
            "--requested-by",
            "node-b",
        ])
        .map_err(|err| format!("parse should succeed: {err}"))?;

        match cli.command {
            Command::Ha(ha) => match ha.command {
                HaCommand::Switchover(switchover) => match switchover.command {
                    SwitchoverCommand::Request(request) => {
                        assert_eq!(request.requested_by, "node-b");
                        Ok(())
                    }
                    _ => Err("expected switchover request".to_string()),
                },
                _ => Err("expected switchover command".to_string()),
            },
        }
    }

    #[test]
    fn parse_env_token_fallbacks() -> Result<(), String> {
        let read_var = "PGTUSKMASTER_READ_TOKEN";
        let admin_var = "PGTUSKMASTER_ADMIN_TOKEN";

        std::env::set_var(read_var, "reader");
        std::env::set_var(admin_var, "admin");

        let parsed = Cli::try_parse_from(["pgtuskmasterctl", "ha", "state"])
            .map_err(|err| format!("parse should succeed: {err}"));

        std::env::remove_var(read_var);
        std::env::remove_var(admin_var);

        let cli = parsed?;
        assert_eq!(cli.read_token.as_deref(), Some("reader"));
        assert_eq!(cli.admin_token.as_deref(), Some("admin"));
        Ok(())
    }
}


===== tests/ha/support/observer.rs =====
use pgtuskmaster_rust::{api::HaPhaseResponse, api::HaStateResponse, state::WorkerError};

#[derive(Clone, Default, serde::Serialize)]
pub struct HaObservationStats {
    pub sample_count: u64,
    pub api_error_count: u64,
    pub max_concurrent_primaries: usize,
    pub leader_change_count: u64,
    pub failsafe_sample_count: u64,
    pub recent_samples: Vec<String>,
}

#[derive(Clone, Copy)]
pub struct HaObserverConfig {
    pub min_successful_samples: u64,
    pub ring_capacity: usize,
}

pub struct HaInvariantObserver {
    config: HaObserverConfig,
    stats: HaObservationStats,
    poll_attempts: u64,
    poll_errors: u64,
    last_poll_error: Option<String>,
    last_leader_signature: Option<String>,
}

impl HaInvariantObserver {
    pub fn new(config: HaObserverConfig) -> Self {
        Self {
            config,
            stats: HaObservationStats::default(),
            poll_attempts: 0,
            poll_errors: 0,
            last_poll_error: None,
            last_leader_signature: None,
        }
    }

    pub fn record_poll_attempt(&mut self) {
        self.poll_attempts = self.poll_attempts.saturating_add(1);
    }

    pub fn record_api_states(
        &mut self,
        states: &[HaStateResponse],
        errors: &[String],
    ) -> Result<(), WorkerError> {
        self.stats.api_error_count = self
            .stats
            .api_error_count
            .saturating_add(len_to_u64(errors.len()));

        if states.is_empty() {
            if !errors.is_empty() {
                self.push_recent(format!("api_blind_spot: {}", errors.join(" | ")));
            }
            return Ok(());
        }

        self.stats.sample_count = self.stats.sample_count.saturating_add(1);
        let primary_count = states
            .iter()
            .filter(|state| state.ha_phase == HaPhaseResponse::Primary)
            .count();
        self.stats.max_concurrent_primaries =
            self.stats.max_concurrent_primaries.max(primary_count);

        let mut leaders = states
            .iter()
            .filter_map(|state| state.leader.clone())
            .collect::<Vec<_>>();
        leaders.sort();
        leaders.dedup();
        let leader_signature = leaders.join("|");
        if self
            .last_leader_signature
            .as_deref()
            .map(|prior| prior != leader_signature.as_str())
            .unwrap_or(false)
        {
            self.stats.leader_change_count = self.stats.leader_change_count.saturating_add(1);
        }
        self.last_leader_signature = Some(leader_signature);

        if states
            .iter()
            .all(|state| state.ha_phase == HaPhaseResponse::FailSafe)
        {
            self.stats.failsafe_sample_count = self.stats.failsafe_sample_count.saturating_add(1);
        }

        let mut fragments = states
            .iter()
            .map(|state| {
                let leader = state.leader.as_deref().unwrap_or("none");
                format!(
                    "{}:{}:leader={leader}",
                    state.self_member_id, state.ha_phase
                )
            })
            .collect::<Vec<_>>();
        fragments.extend(errors.iter().map(|error| format!("api_error={error}")));
        self.push_recent(fragments.join(", "));

        if primary_count > 1 {
            return Err(WorkerError::Message(format!(
                "split-brain detected: more than one primary; observations={} errors={}",
                states
                    .iter()
                    .map(|state| format!("{}:{}", state.self_member_id, state.ha_phase))
                    .collect::<Vec<_>>()
                    .join(","),
                summarize_errors(errors)
            )));
        }

        Ok(())
    }

    pub fn record_sql_roles(
        &mut self,
        roles: &[(String, String)],
        errors: &[String],
    ) -> Result<(), WorkerError> {
        if roles.is_empty() {
            if !errors.is_empty() {
                self.push_recent(format!("sql_blind_spot: {}", errors.join(" | ")));
            }
            return Ok(());
        }

        self.stats.sample_count = self.stats.sample_count.saturating_add(1);
        let primary_count = roles.iter().filter(|(_, role)| role == "primary").count();
        self.stats.max_concurrent_primaries =
            self.stats.max_concurrent_primaries.max(primary_count);

        let mut fragments = roles
            .iter()
            .map(|(node_id, role)| format!("{node_id}:{role}"))
            .collect::<Vec<_>>();
        fragments.extend(errors.iter().map(|error| format!("sql_error={error}")));
        self.push_recent(format!("sql_roles=[{}]", fragments.join(", ")));

        if primary_count > 1 {
            return Err(WorkerError::Message(format!(
                "split-brain detected via SQL roles: observations={} errors={}",
                roles
                    .iter()
                    .map(|(node_id, role)| format!("{node_id}:{role}"))
                    .collect::<Vec<_>>()
                    .join(","),
                summarize_errors(errors)
            )));
        }

        Ok(())
    }

    pub fn record_observation_gap(&mut self, api_errors: &[String], sql_errors: &[String]) {
        self.poll_errors = self.poll_errors.saturating_add(1);
        let message = format!(
            "api_errors={}; sql_errors={}",
            summarize_errors(api_errors),
            summarize_errors(sql_errors)
        );
        self.last_poll_error = Some(message.clone());
        self.push_recent(format!("observation_gap:{message}"));
    }

    pub fn record_transport_error(&mut self, error: impl Into<String>) {
        self.poll_errors = self.poll_errors.saturating_add(1);
        let message = error.into();
        self.last_poll_error = Some(message.clone());
        self.push_recent(format!("transport_error:{message}"));
    }

    pub fn stats(&self) -> &HaObservationStats {
        &self.stats
    }

    pub fn into_stats(self) -> HaObservationStats {
        self.stats
    }

    pub fn finalize_no_dual_primary_window(&self) -> Result<(), WorkerError> {
        if self.stats.sample_count < self.config.min_successful_samples {
            let detail = self.last_poll_error.as_deref().unwrap_or("none");
            return Err(WorkerError::Message(format!(
                "insufficient evidence for split-brain window assertion: successful_samples={} min_successful_samples={} poll_attempts={} poll_errors={} last_poll_error={} recent_samples={}",
                self.stats.sample_count,
                self.config.min_successful_samples,
                self.poll_attempts,
                self.poll_errors,
                detail,
                summarize_recent_samples(&self.stats.recent_samples),
            )));
        }

        assert_no_dual_primary_in_samples(self.stats(), self.config.min_successful_samples)
    }

    fn push_recent(&mut self, sample: String) {
        if self.config.ring_capacity == 0 {
            return;
        }
        if self.stats.recent_samples.len() >= self.config.ring_capacity {
            let _ = self.stats.recent_samples.remove(0);
        }
        self.stats.recent_samples.push(sample);
    }
}

pub fn assert_no_dual_primary_in_samples(
    stats: &HaObservationStats,
    min_successful_samples: u64,
) -> Result<(), WorkerError> {
    if stats.sample_count < min_successful_samples {
        return Err(WorkerError::Message(format!(
            "insufficient HA sample evidence: sample_count={} min_successful_samples={} api_error_count={} recent_samples={}",
            stats.sample_count,
            min_successful_samples,
            stats.api_error_count,
            summarize_recent_samples(&stats.recent_samples),
        )));
    }
    if stats.max_concurrent_primaries > 1 {
        return Err(WorkerError::Message(format!(
            "dual primary observed during sampled window; max_concurrent_primaries={} recent_samples={}",
            stats.max_concurrent_primaries,
            summarize_recent_samples(&stats.recent_samples),
        )));
    }
    Ok(())
}

fn len_to_u64(value: usize) -> u64 {
    u64::try_from(value)
        .ok()
        .map_or(u64::MAX, core::convert::identity)
}

fn summarize_errors(errors: &[String]) -> String {
    if errors.is_empty() {
        "none".to_string()
    } else {
        errors.join(" | ")
    }
}

fn summarize_recent_samples(samples: &[String]) -> String {
    if samples.is_empty() {
        "none".to_string()
    } else {
        samples.join(" || ")
    }
}

#[cfg(test)]
mod unit_tests {
    use super::{
        assert_no_dual_primary_in_samples, HaInvariantObserver, HaObservationStats,
        HaObserverConfig,
    };
    use pgtuskmaster_rust::api::{
        DcsTrustResponse, HaDecisionResponse, HaPhaseResponse, HaStateResponse,
    };

    type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn ha_state(member_id: &str, phase: HaPhaseResponse, leader: Option<&str>) -> HaStateResponse {
        HaStateResponse {
            cluster_name: "cluster-e2e".to_string(),
            scope: "scope-ha-e2e".to_string(),
            self_member_id: member_id.to_string(),
            leader: leader.map(ToString::to_string),
            switchover_requested_by: None,
            member_count: 3,
            dcs_trust: DcsTrustResponse::FullQuorum,
            ha_phase: phase,
            ha_tick: 1,
            ha_decision: HaDecisionResponse::NoChange,
            snapshot_sequence: 1,
        }
    }

    #[test]
    fn zero_sample_finalization_fails_closed() -> TestResult {
        let observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity: 4,
        });
        let result = observer.finalize_no_dual_primary_window();
        if result.is_ok() {
            return Err(Box::new(std::io::Error::other(
                "expected finalization to fail with zero samples",
            )));
        }
        Ok(())
    }

    #[test]
    fn insufficient_sample_threshold_fails() -> TestResult {
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 2,
            ring_capacity: 4,
        });
        observer.record_poll_attempt();
        observer.record_api_states(
            &[ha_state("node-1", HaPhaseResponse::Primary, Some("node-1"))],
            &[],
        )?;
        let result = observer.finalize_no_dual_primary_window();
        if result.is_ok() {
            return Err(Box::new(std::io::Error::other(
                "expected finalization to fail when the minimum sample threshold is not met",
            )));
        }
        Ok(())
    }

    #[test]
    fn successful_finalization_with_enough_samples() -> TestResult {
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 2,
            ring_capacity: 4,
        });
        for _ in 0..2 {
            observer.record_poll_attempt();
            observer.record_api_states(
                &[ha_state("node-1", HaPhaseResponse::Primary, Some("node-1"))],
                &[],
            )?;
        }
        observer.finalize_no_dual_primary_window()?;
        Ok(())
    }

    #[test]
    fn dual_primary_sample_is_rejected() -> TestResult {
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity: 4,
        });
        observer.record_poll_attempt();
        let result = observer.record_api_states(
            &[
                ha_state("node-1", HaPhaseResponse::Primary, Some("node-1")),
                ha_state("node-2", HaPhaseResponse::Primary, Some("node-2")),
            ],
            &[],
        );
        if result.is_ok() {
            return Err(Box::new(std::io::Error::other(
                "expected dual-primary sample to fail",
            )));
        }
        Ok(())
    }

    #[test]
    fn standalone_sample_assertion_rejects_dual_primary_stats() -> TestResult {
        let stats = HaObservationStats {
            sample_count: 3,
            api_error_count: 1,
            max_concurrent_primaries: 2,
            leader_change_count: 0,
            failsafe_sample_count: 0,
            recent_samples: vec!["node-1:Primary,node-2:Primary".to_string()],
        };
        let result = assert_no_dual_primary_in_samples(&stats, 1);
        if result.is_ok() {
            return Err(Box::new(std::io::Error::other(
                "expected dual-primary stats assertion to fail",
            )));
        }
        Ok(())
    }

    #[test]
    fn observer_transport_and_stats_helpers_are_reachable() {
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity: 4,
        });
        observer.record_transport_error("synthetic");
        let stats = observer.into_stats();
        assert_eq!(stats.recent_samples.len(), 1);
    }
}


===== docs/tmp/verbose_extra_context/cluster-start-command.md =====
# Extra Context: Minimum Local Command To Start The Three-Node HA Cluster

K2 asked for the minimum docker compose command to start the three-node cluster on a local machine without custom networking. This note answers that question using only source-backed details from this repository.

## What the compose file itself requires

The cluster compose file at `docker/compose/docker-compose.cluster.yml` defines four services:

- `etcd`
- `node-a`
- `node-b`
- `node-c`

It also defines a normal bridge network named `pgtm-internal`, so the cluster does not require the operator to create a custom Docker network up front. The service-to-service hostnames used in the runtime configs are `etcd`, `node-a`, `node-b`, and `node-c`, and those names come from the compose service names.

The compose file depends on environment-variable substitution for:

- `PGTUSKMASTER_IMAGE`
- `ETCD_IMAGE`
- `PGTM_SECRET_SUPERUSER_FILE`
- `PGTM_SECRET_REPLICATOR_FILE`
- `PGTM_SECRET_REWINDER_FILE`
- `PGTM_CLUSTER_NODE_A_API_PORT`
- `PGTM_CLUSTER_NODE_A_PG_PORT`
- `PGTM_CLUSTER_NODE_B_API_PORT`
- `PGTM_CLUSTER_NODE_B_PG_PORT`
- `PGTM_CLUSTER_NODE_C_API_PORT`
- `PGTM_CLUSTER_NODE_C_PG_PORT`

That means a plain `docker compose -f docker/compose/docker-compose.cluster.yml up -d` is not enough on its own unless the caller has already exported all required variables in the shell.

## The repository-provided env file

The repository ships `.env.docker.example`, which defines all of the variables the cluster compose file needs for a local example run.

Important details from `.env.docker.example`:

- `PGTUSKMASTER_IMAGE=pgtuskmaster:local`
- `ETCD_IMAGE=quay.io/coreos/etcd:v3.5.21`
- `PGTM_SECRET_SUPERUSER_FILE=../secrets/postgres-superuser.password.example`
- `PGTM_SECRET_REPLICATOR_FILE=../secrets/replicator.password.example`
- `PGTM_SECRET_REWINDER_FILE=../secrets/rewinder.password.example`
- `PGTM_CLUSTER_NODE_A_API_PORT=18081`
- `PGTM_CLUSTER_NODE_A_PG_PORT=15433`
- `PGTM_CLUSTER_NODE_B_API_PORT=18082`
- `PGTM_CLUSTER_NODE_B_PG_PORT=15434`
- `PGTM_CLUSTER_NODE_C_API_PORT=18083`
- `PGTM_CLUSTER_NODE_C_PG_PORT=15435`

Those secret paths are written relative to the compose file directory, so they resolve to the example password files under `docker/secrets/`.

## The safest documented command to recommend

The most source-backed command to recommend in a tutorial is:

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d --build
```

Reasons this is the safest recommendation:

- It supplies all required variable substitutions through a checked-in env file.
- It uses the checked-in compose file directly.
- It does not require any custom Docker network preparation because the compose file already defines `pgtm-internal`.
- It matches the repository's own smoke flow shape, which starts the cluster with `docker compose ... up -d --build` after generating a compatible env file.

## Strict minimum versus operationally safe minimum

There are two slightly different ways to phrase "minimum":

1. Strict compose invocation minimum when the image is already available locally:

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d
```

2. Safe tutorial command that also covers the common "I have not built the image yet" case:

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d --build
```

For a first-run tutorial, the second command is more defensible because the compose file points each node service at the repo root build context and `docker/Dockerfile.prod`.

## What that command publishes on the host

With `.env.docker.example`, the cluster exposes:

- `node-a` API on `127.0.0.1:18081`
- `node-a` PostgreSQL on `127.0.0.1:15433`
- `node-b` API on `127.0.0.1:18082`
- `node-b` PostgreSQL on `127.0.0.1:15434`
- `node-c` API on `127.0.0.1:18083`
- `node-c` PostgreSQL on `127.0.0.1:15435`

That makes the tutorial concrete because the operator can immediately target node-a's API with `http://127.0.0.1:18081`.

## Evidence sources behind this note

- `docker/compose/docker-compose.cluster.yml`
- `.env.docker.example`
- `tools/docker/smoke-cluster.sh`
- `tools/docker/common.sh`


===== docs/tmp/verbose_extra_context/leader-check-command.md =====
# Extra Context: Exact pgtuskmasterctl Command To Check The Current Leader

K2 asked for the exact `pgtuskmasterctl` command to check which node is the current leader. This note answers that with source-backed detail from the CLI parser, client, and output renderer.

## There is no dedicated leader subcommand

The current CLI parser in `src/cli/args.rs` exposes:

- `pgtuskmasterctl ha state`
- `pgtuskmasterctl ha switchover clear`
- `pgtuskmasterctl ha switchover request --requested-by <member>`

There is no `ha leader get`, `ha leader show`, or `ha leader set` command in the current CLI shape.

This matters because `tests/cli_binary.rs` intentionally invokes `pgtuskmasterctl ha leader set` and asserts that it exits with clap usage code `2`. That test is strong evidence that older leader-specific command wording should not be used in fresh docs.

## How leader information is retrieved

`src/cli/mod.rs` routes `ha state` to `CliApiClient::get_ha_state()`.

`src/cli/client.rs` shows that `get_ha_state()` performs:

- HTTP method: `GET`
- path: `/ha/state`

So the CLI does not ask a separate leader endpoint. It reads the full HA state payload and extracts the `leader` field from that response.

`src/api/mod.rs` defines the returned shape. `HaStateResponse` includes:

- `self_member_id`
- `leader`
- `member_count`
- `dcs_trust`
- `ha_phase`
- `ha_decision`
- `snapshot_sequence`

The current leader is therefore whatever value appears in the `leader` field.

## Exact command to run against node-a in the example cluster

Using the fixed API port from `.env.docker.example`, the most explicit text-output command is:

```bash
pgtuskmasterctl --base-url http://127.0.0.1:18081 --output text ha state
```

That is the best exact command for the tutorial because:

- it names the binary exactly as defined by the clap parser
- it targets node-a's published API port from the checked-in example env file
- it asks for text output, which makes the `leader=` line easy to read in a tutorial

The rendered text output includes a line in the form:

```text
leader=node-a
```

or:

```text
leader=node-b
```

or:

```text
leader=<none>
```

That format is backed by `src/cli/output.rs`, which prints the `leader` field as a standalone `leader=...` line in text mode.

## JSON alternative

If the tutorial wants machine-readable output instead, the equivalent command is:

```bash
pgtuskmasterctl --base-url http://127.0.0.1:18081 ha state
```

JSON is the default output mode, so omitting `--output` still works. In that case, the current leader is the value of the JSON `leader` field.

## Authentication note for this docker cluster

All three cluster runtime configs set:

- API TLS mode: disabled
- API auth type: disabled

That means the example docker cluster does not require `--read-token` or `--admin-token` for the `ha state` read command.

## Evidence sources behind this note

- `src/cli/args.rs`
- `src/cli/mod.rs`
- `src/cli/client.rs`
- `src/cli/output.rs`
- `src/api/mod.rs`
- `.env.docker.example`
- `tests/cli_binary.rs`
- `docker/configs/cluster/node-a/runtime.toml`
- `docker/configs/cluster/node-b/runtime.toml`
- `docker/configs/cluster/node-c/runtime.toml`
