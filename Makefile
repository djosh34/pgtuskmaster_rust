MDBOOK := .tools/mdbook/bin/mdbook
MDBOOK_MERMAID := .tools/mdbook/bin/mdbook-mermaid
DOCKER_SINGLE_COMPOSE_FILE := docker/compose/docker-compose.single.yml
DOCKER_CLUSTER_COMPOSE_FILE := docker/compose/docker-compose.cluster.yml
DOCKER_ENV_FILE ?= .env.docker
DOCKER_SINGLE_PROJECT ?= pgtuskmaster-single
DOCKER_CLUSTER_PROJECT ?= pgtuskmaster-cluster

SHELL := /usr/bin/env bash

.PHONY: check test test-long lint lint.no_silent_errors docs-build docs-serve docs-hygiene docs-lint ensure-docker ensure-mdbook ensure-mdbook-mermaid ensure-node ensure-nextest ensure-timeout guard-makeflags docker-compose-config docker-up docker-down docker-up-cluster docker-status-cluster docker-down-cluster docker-smoke-single docker-smoke-cluster

SINGLE_DASH_MAKEFLAGS := $(filter -%,$(MAKEFLAGS))
SINGLE_DASH_MAKEFLAGS := $(filter-out --%,$(SINGLE_DASH_MAKEFLAGS))

ifneq ($(filter -n --dry-run --just-print --recon n,$(MAKEFLAGS))$(filter %n%,$(SINGLE_DASH_MAKEFLAGS)),)
$(error Refusing to run Makefile gates with dry-run enabled (MAKEFLAGS contains -n/--dry-run))
endif
ifneq ($(filter -i --ignore-errors i,$(MAKEFLAGS))$(filter %i%,$(SINGLE_DASH_MAKEFLAGS)),)
$(error Refusing to run Makefile gates with ignore-errors enabled (MAKEFLAGS contains -i/--ignore-errors))
endif
ifneq ($(filter -e --environment-overrides e,$(MAKEFLAGS))$(filter %e%,$(SINGLE_DASH_MAKEFLAGS)),)
$(error Refusing to run Makefile gates with environment overrides enabled (MAKEFLAGS contains -e/--environment-overrides))
endif

# The workspace mount this repo typically lives on can exhibit intermittent
# linker/archive flake with incremental artifacts. Disable incremental builds by
# default for deterministic `make` gates; override with `CARGO_INCREMENTAL=1`
# if you explicitly want it.
CARGO_INCREMENTAL ?= 0
CARGO_GATE_TARGET_DIR := /tmp/pgtuskmaster_rust-target
ifeq ($(CARGO_INCREMENTAL),1)
CARGO_INCREMENTAL_BOOL := true
else
CARGO_INCREMENTAL_BOOL := false
endif
ifneq ($(origin TIMEOUT_BIN),undefined)
$(error TIMEOUT_BIN must not be set externally; edit Makefile to change timeout resolution)
endif
TIMEOUT_BIN := $(shell command -v timeout 2>/dev/null || command -v gtimeout 2>/dev/null)

CHECK_TIMEOUT_SECS := 300
CHECK_TIMEOUT_KILL_AFTER_SECS := 15

LINT_DOCS_TIMEOUT_SECS := 120
LINT_DOCS_TIMEOUT_KILL_AFTER_SECS := 15

LINT_CLIPPY_TIMEOUT_SECS := 1200
LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS := 30
DOCKER_CONFIG_TIMEOUT_SECS := 120
DOCKER_CONFIG_TIMEOUT_KILL_AFTER_SECS := 15
DOCKER_SMOKE_SINGLE_TIMEOUT_SECS := 600
DOCKER_SMOKE_SINGLE_TIMEOUT_KILL_AFTER_SECS := 30
DOCKER_SMOKE_CLUSTER_TIMEOUT_SECS := 900
DOCKER_SMOKE_CLUSTER_TIMEOUT_KILL_AFTER_SECS := 30

GATE_RUN_ID := $(shell date -u +%Y%m%dT%H%M%SZ)-$(shell printf '%s' "$$PPID")-$(shell printf '%s' "$$RANDOM")
GATE_EVIDENCE_DIR := $(CURDIR)/.ralph/evidence/gates/$(GATE_RUN_ID)

GATE_STEP := $(CURDIR)/tools/gate-step.sh

ifneq ($(origin CHECK_TIMEOUT_SECS),file)
$(error CHECK_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin CHECK_TIMEOUT_KILL_AFTER_SECS),file)
$(error CHECK_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin LINT_DOCS_TIMEOUT_SECS),file)
$(error LINT_DOCS_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin LINT_DOCS_TIMEOUT_KILL_AFTER_SECS),file)
$(error LINT_DOCS_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin LINT_CLIPPY_TIMEOUT_SECS),file)
$(error LINT_CLIPPY_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS),file)
$(error LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin DOCKER_CONFIG_TIMEOUT_SECS),file)
$(error DOCKER_CONFIG_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin DOCKER_CONFIG_TIMEOUT_KILL_AFTER_SECS),file)
$(error DOCKER_CONFIG_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin DOCKER_SMOKE_SINGLE_TIMEOUT_SECS),file)
$(error DOCKER_SMOKE_SINGLE_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin DOCKER_SMOKE_SINGLE_TIMEOUT_KILL_AFTER_SECS),file)
$(error DOCKER_SMOKE_SINGLE_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin DOCKER_SMOKE_CLUSTER_TIMEOUT_SECS),file)
$(error DOCKER_SMOKE_CLUSTER_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin DOCKER_SMOKE_CLUSTER_TIMEOUT_KILL_AFTER_SECS),file)
$(error DOCKER_SMOKE_CLUSTER_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin TIMEOUT_BIN),file)
$(error TIMEOUT_BIN must not be set externally; edit Makefile to change timeout resolution)
endif
ifneq ($(origin GATE_RUN_ID),file)
$(error GATE_RUN_ID must not be set externally; edit Makefile to change evidence run id generation)
endif
ifneq ($(origin GATE_EVIDENCE_DIR),file)
$(error GATE_EVIDENCE_DIR must not be set externally; edit Makefile to change gate evidence directory)
endif
ifneq ($(origin GATE_STEP),file)
$(error GATE_STEP must not be set externally; edit Makefile to change gate step runner path)
endif

guard-makeflags:
	@set -eu; \
	flags=" $${MAKEFLAGS:-} "; \
	printf '%s\n' "$${flags}" | grep -Eq '(^|[[:space:]])(-n|--just-print|--dry-run|--recon|n)([[:space:]]|$$)' && { \
		echo "Refusing to run gate target with MAKEFLAGS dry-run enabled (-n/--dry-run)." >&2; \
		exit 1; \
	}; \
	printf '%s\n' "$${flags}" | grep -Eq '(^|[[:space:]])(-i|--ignore-errors|i)([[:space:]]|$$)' && { \
		echo "Refusing to run gate target with MAKEFLAGS ignore-errors enabled (-i/--ignore-errors)." >&2; \
		exit 1; \
	}; \
	:

ensure-mdbook:
	@test -x "$(MDBOOK)" || (echo "missing mdBook binary: run ./tools/install-mdbook.sh" >&2; exit 1)

ensure-mdbook-mermaid: ensure-mdbook
	@test -x "$(MDBOOK_MERMAID)" || (echo "missing mdbook-mermaid binary: run ./tools/install-mdbook-mermaid.sh" >&2; exit 1)

ensure-node:
	@command -v node >/dev/null 2>&1 || (echo "missing node binary (required for Mermaid docs lint)" >&2; exit 1)

ensure-nextest:
	@command -v cargo-nextest >/dev/null 2>&1 || (echo "missing cargo-nextest binary: run ./tools/install-cargo-nextest.sh" >&2; exit 1)

ensure-timeout:
	@test -n "$(TIMEOUT_BIN)" || (echo "missing timeout binary (install coreutils). Need either 'timeout' (Linux) or 'gtimeout' (macOS)." >&2; exit 1)

ensure-docker:
	@command -v docker >/dev/null 2>&1 || (echo "missing docker binary" >&2; exit 1)
	@docker info >/dev/null 2>&1 || (echo "docker daemon is not reachable" >&2; exit 1)
	@docker compose version >/dev/null 2>&1 || (echo "docker compose plugin is not available" >&2; exit 1)

check:
	"$(GATE_STEP)" --gate check --step check.cargo_check --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(CHECK_TIMEOUT_SECS)" --kill-after-secs "$(CHECK_TIMEOUT_KILL_AFTER_SECS)" -- cargo check --all-targets --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)"

check: guard-makeflags ensure-timeout

test: guard-makeflags ensure-nextest
	@set -euo pipefail; \
	status=0; \
	cargo nextest run --workspace --all-targets --profile default --no-fail-fast --no-tests fail --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)" || status="$$?"; \
	python3 ./tools/export-nextest-junit-logs.py ./target/nextest/default/junit.xml ./target/nextest/default/logs; \
	exit "$$status"

test-long: guard-makeflags ensure-nextest ensure-timeout ensure-docker
	@echo "test-long runs the ultra-long HA scenarios plus Docker Compose validation and smoke coverage."
	@set -euo pipefail; \
	status=0; \
	cargo nextest run --workspace --all-targets --profile ultra-long --no-fail-fast --no-tests fail --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)" || status="$$?"; \
	python3 ./tools/export-nextest-junit-logs.py ./target/nextest/ultra-long/junit.xml ./target/nextest/ultra-long/logs; \
	if [[ "$$status" -ne 0 ]]; then \
		exit "$$status"; \
	fi
	"$(GATE_STEP)" --gate test-long --step test_long.compose_config --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(DOCKER_CONFIG_TIMEOUT_SECS)" --kill-after-secs "$(DOCKER_CONFIG_TIMEOUT_KILL_AFTER_SECS)" -- \
		./tools/docker/compose-config-check.sh
	"$(GATE_STEP)" --gate test-long --step test_long.docker_smoke_single --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(DOCKER_SMOKE_SINGLE_TIMEOUT_SECS)" --kill-after-secs "$(DOCKER_SMOKE_SINGLE_TIMEOUT_KILL_AFTER_SECS)" -- \
		./tools/docker/smoke-single.sh
	"$(GATE_STEP)" --gate test-long --step test_long.docker_smoke_cluster --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(DOCKER_SMOKE_CLUSTER_TIMEOUT_SECS)" --kill-after-secs "$(DOCKER_SMOKE_CLUSTER_TIMEOUT_KILL_AFTER_SECS)" -- \
		./tools/docker/smoke-cluster.sh

docs-lint: guard-makeflags ensure-timeout ensure-node
	@echo "gate evidence: $(GATE_EVIDENCE_DIR)"
	"$(GATE_STEP)" --gate docs-lint --step docs_lint.mermaid --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_DOCS_TIMEOUT_SECS)" --kill-after-secs "$(LINT_DOCS_TIMEOUT_KILL_AFTER_SECS)" -- \
		node ./tools/docs-mermaid-lint.mjs
	"$(GATE_STEP)" --gate docs-lint --step docs_lint.no_code_guard --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_DOCS_TIMEOUT_SECS)" --kill-after-secs "$(LINT_DOCS_TIMEOUT_KILL_AFTER_SECS)" -- \
		./tools/docs-architecture-no-code-guard.sh

lint.no_silent_errors: guard-makeflags ensure-timeout
	"$(GATE_STEP)" --gate lint --step lint.no_silent_errors --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_DOCS_TIMEOUT_SECS)" --kill-after-secs "$(LINT_DOCS_TIMEOUT_KILL_AFTER_SECS)" -- \
		./tools/lint-no-silent-errors.sh

lint: guard-makeflags ensure-timeout docs-lint lint.no_silent_errors
	"$(GATE_STEP)" --gate lint --step lint.clippy.all_targets --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_CLIPPY_TIMEOUT_SECS)" --kill-after-secs "$(LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS)" -- \
		cargo clippy --all-targets --all-features --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)" -- -D warnings
	# Strict restriction-lint pass for runtime library builds.
	"$(GATE_STEP)" --gate lint --step lint.clippy.lib_restrictions --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_CLIPPY_TIMEOUT_SECS)" --kill-after-secs "$(LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS)" -- \
		cargo clippy --lib --all-features --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)" -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
	# Strict restriction-lint pass for test targets.
	"$(GATE_STEP)" --gate lint --step lint.clippy.tests_restrictions --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_CLIPPY_TIMEOUT_SECS)" --kill-after-secs "$(LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS)" -- \
		cargo clippy --tests --all-features --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)" -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
	# Skeptical all-target guard so restrictions are enforced uniformly.
	"$(GATE_STEP)" --gate lint --step lint.clippy.all_targets_restrictions --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_CLIPPY_TIMEOUT_SECS)" --kill-after-secs "$(LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS)" -- \
		cargo clippy --all-targets --all-features --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)" -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented

docs-build: ensure-mdbook-mermaid
	PATH="$(CURDIR)/.tools/mdbook/bin:$$PATH" "$(MDBOOK)" build docs

docs-serve: ensure-mdbook-mermaid
	PATH="$(CURDIR)/.tools/mdbook/bin:$$PATH" "$(MDBOOK)" serve docs -n 127.0.0.1 -p 3000

docs-hygiene:
	@set -euo pipefail; \
	tracked="$$(git ls-files -- docs/book docs/.mdbook)"; \
	if [[ -n "$${tracked}" ]]; then \
		echo "generated docs output is tracked (must be removed from git index):" >&2; \
		echo "$${tracked}" >&2; \
		exit 1; \
	fi

docker-compose-config: guard-makeflags ensure-timeout ensure-docker
	"$(GATE_STEP)" --gate docker --step docker.compose_config --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(DOCKER_CONFIG_TIMEOUT_SECS)" --kill-after-secs "$(DOCKER_CONFIG_TIMEOUT_KILL_AFTER_SECS)" -- \
		./tools/docker/compose-config-check.sh

docker-up: guard-makeflags ensure-docker
	@test -f "$(DOCKER_ENV_FILE)" || (echo "missing $(DOCKER_ENV_FILE); copy .env.docker.example and point it at real secret files first" >&2; exit 1)
	docker compose --project-name "$(DOCKER_SINGLE_PROJECT)" --env-file "$(DOCKER_ENV_FILE)" -f "$(DOCKER_SINGLE_COMPOSE_FILE)" up -d --build

docker-down: guard-makeflags ensure-docker
	@test -f "$(DOCKER_ENV_FILE)" || (echo "missing $(DOCKER_ENV_FILE); nothing to tear down" >&2; exit 1)
	docker compose --project-name "$(DOCKER_SINGLE_PROJECT)" --env-file "$(DOCKER_ENV_FILE)" -f "$(DOCKER_SINGLE_COMPOSE_FILE)" down -v --remove-orphans

docker-up-cluster: guard-makeflags ensure-docker
	./tools/docker/cluster.sh up --env-file "$(DOCKER_ENV_FILE)" --project-name "$(DOCKER_CLUSTER_PROJECT)"

docker-status-cluster: guard-makeflags ensure-docker
	./tools/docker/cluster.sh status --env-file "$(DOCKER_ENV_FILE)" --project-name "$(DOCKER_CLUSTER_PROJECT)"

docker-down-cluster: guard-makeflags ensure-docker
	./tools/docker/cluster.sh down --env-file "$(DOCKER_ENV_FILE)" --project-name "$(DOCKER_CLUSTER_PROJECT)"

docker-smoke-single: guard-makeflags ensure-timeout ensure-docker
	"$(GATE_STEP)" --gate docker --step docker.smoke_single --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(DOCKER_SMOKE_SINGLE_TIMEOUT_SECS)" --kill-after-secs "$(DOCKER_SMOKE_SINGLE_TIMEOUT_KILL_AFTER_SECS)" -- \
		./tools/docker/smoke-single.sh

docker-smoke-cluster: guard-makeflags ensure-timeout ensure-docker
	"$(GATE_STEP)" --gate docker --step docker.smoke_cluster --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(DOCKER_SMOKE_CLUSTER_TIMEOUT_SECS)" --kill-after-secs "$(DOCKER_SMOKE_CLUSTER_TIMEOUT_KILL_AFTER_SECS)" -- \
		./tools/docker/smoke-cluster.sh
