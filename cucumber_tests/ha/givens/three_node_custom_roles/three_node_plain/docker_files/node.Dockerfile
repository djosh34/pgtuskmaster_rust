FROM postgres:16-bookworm

USER root

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates iproute2 iptables procps \
    && rm -rf /var/lib/apt/lists/* \
    && install -d -o postgres -g postgres \
        /etc/pgtuskmaster \
        /etc/pgtuskmaster/tls \
        /usr/local/lib/pgtuskmaster/wrappers \
        /var/lib/pgtuskmaster/socket \
        /var/lib/pgtuskmaster/faults \
        /var/log/pgtuskmaster

COPY docker_files/bin/pgtuskmaster /usr/local/bin/pgtuskmaster
COPY docker_files/wrappers /usr/local/lib/pgtuskmaster/wrappers

RUN chmod 0755 /usr/local/bin/pgtuskmaster \
    && chmod 0755 /usr/local/lib/pgtuskmaster/wrappers/*

USER postgres

WORKDIR /var/lib/postgresql

ENTRYPOINT ["/usr/local/bin/pgtuskmaster", "--config", "/etc/pgtuskmaster/runtime.toml"]
