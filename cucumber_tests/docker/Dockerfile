# syntax=docker/dockerfile:1.7

ARG RUST_IMAGE=rust:1.93-bookworm

FROM ${RUST_IMAGE} AS chef

WORKDIR /app

RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        ca-certificates \
        pkg-config \
        protobuf-compiler \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-chef --locked

FROM chef AS planner

COPY Cargo.toml Cargo.lock ./

# Create a minimal target skeleton so cargo-chef can derive the dependency recipe
# without copying the real src tree before the cacheable dependency layer.
RUN install -d src/bin \
    && printf '%s\n' '// cargo-chef skeleton target' > src/lib.rs \
    && printf '%s\n' 'fn main() {}' > src/bin/pgtuskmaster.rs \
    && printf '%s\n' 'fn main() {}' > src/bin/pgtm.rs \
    && cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --locked --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked --bin pgtuskmaster --bin pgtm \
    && install -Dm0755 /app/target/release/pgtuskmaster /out/pgtuskmaster \
    && install -Dm0755 /app/target/release/pgtm /out/pgtm

FROM postgres:16-bookworm AS runtime

USER root

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates iproute2 iptables procps \
    && rm -rf /var/lib/apt/lists/* \
    && install -d -o postgres -g postgres \
        /etc/pgtuskmaster \
        /etc/pgtuskmaster/observer \
        /etc/pgtuskmaster/tls \
        /usr/local/lib/pgtuskmaster/wrappers \
        /var/lib/pgtuskmaster/socket \
        /var/lib/pgtuskmaster/faults \
        /var/log/pgtuskmaster \
        /run/secrets

COPY --from=builder /out/pgtuskmaster /usr/local/bin/pgtuskmaster
COPY --from=builder /out/pgtm /usr/local/bin/pgtm
COPY cucumber_tests/docker/wrappers /usr/local/lib/pgtuskmaster/wrappers

RUN chmod 0755 /usr/local/bin/pgtuskmaster /usr/local/bin/pgtm \
    && chmod 0755 /usr/local/lib/pgtuskmaster/wrappers/*

USER postgres

WORKDIR /var/lib/postgresql

ENTRYPOINT ["/usr/local/bin/pgtuskmaster", "--config", "/etc/pgtuskmaster/runtime.toml"]
