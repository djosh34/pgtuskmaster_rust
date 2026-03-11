FROM postgres:16-bookworm

USER root

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates iproute2 iptables procps \
    && rm -rf /var/lib/apt/lists/* \
    && install -d -o postgres -g postgres \
        /etc/pgtuskmaster \
        /etc/pgtuskmaster/observer \
        /etc/pgtuskmaster/tls \
        /run/secrets

COPY docker_files/bin/pgtm /usr/local/bin/pgtm

RUN chmod 0755 /usr/local/bin/pgtm

USER postgres

WORKDIR /var/lib/postgresql

CMD ["/usr/bin/tail", "-f", "/dev/null"]
