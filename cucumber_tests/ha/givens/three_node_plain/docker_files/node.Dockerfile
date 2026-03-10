FROM postgres:16-bookworm

USER root

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && install -d -o postgres -g postgres \
        /etc/pgtuskmaster \
        /var/lib/pgtuskmaster/socket \
        /var/log/pgtuskmaster

COPY docker_files/bin/pgtuskmaster /usr/local/bin/pgtuskmaster

RUN chmod 0755 /usr/local/bin/pgtuskmaster

USER postgres

WORKDIR /var/lib/postgresql

ENTRYPOINT ["/usr/local/bin/pgtuskmaster", "--config", "/etc/pgtuskmaster/runtime.toml"]
