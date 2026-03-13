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

FROM postgres:16-bookworm AS runtime-base

USER root

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && install -d -o postgres -g postgres \
        /etc/pgtuskmaster \
        /etc/pgtuskmaster/certs \
        /etc/pgtuskmaster/tls \
        /etc/pgtuskmaster/pg \
        /etc/pgtuskmaster/secrets \
        /var/lib/pgtuskmaster/socket \
        /var/log/pgtuskmaster

COPY --from=builder /out/pgtuskmaster /usr/local/bin/pgtuskmaster
COPY --from=builder /out/pgtm /usr/local/bin/pgtm

RUN chmod 0755 /usr/local/bin/pgtuskmaster /usr/local/bin/pgtm

USER postgres

WORKDIR /var/lib/postgresql

ENTRYPOINT ["/usr/local/bin/pgtuskmaster"]
CMD ["--config", "/etc/pgtuskmaster/runtime.toml"]

FROM runtime-base AS runtime

FROM runtime-base AS ha-test

USER root

RUN apt-get update \
    && apt-get install --yes --no-install-recommends iproute2 iptables procps \
    && rm -rf /var/lib/apt/lists/* \
    && install -d -o postgres -g postgres /usr/local/lib/pgtuskmaster/wrappers

COPY tests/docker/entrypoint.sh /usr/local/bin/pgtuskmaster-ha-entrypoint
COPY tests/docker/wrappers /usr/local/lib/pgtuskmaster/wrappers

RUN chmod 0755 /usr/local/bin/pgtuskmaster-ha-entrypoint \
    && chmod 0755 /usr/local/lib/pgtuskmaster/wrappers/*

USER postgres

ENTRYPOINT ["/usr/local/bin/pgtuskmaster-ha-entrypoint"]
