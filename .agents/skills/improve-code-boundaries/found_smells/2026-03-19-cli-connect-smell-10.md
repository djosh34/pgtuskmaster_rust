path: /home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/connect.rs 53-208

I found smell 10:


I think this is smell 10 because the connection rendering path is fragmented into tiny private helpers that mostly forward to the next helper. `resolve_primary_view` and `resolve_replicas_view` own the workflow, but reading them still requires bouncing through `build_connection_view`, `build_connection_target`, `build_connection_conninfo`, and `build_connection_tls`.


code:
```rust
fn resolve_primary_view(
    state: &NodeState,
    queried_via: StateQueryOriginDto,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<StateDerivedConnectionCommandDto, CliError> {
    Ok(build_connection_view(
        state,
        queried_via,
        StateDerivedConnectionCommandKind::Primary,
        vec![build_connection_target(
            primary_id.as_str(),
            member,
            tls,
            emit_tls,
        )?],
    ))
}

fn build_connection_target(
    member_id: &str,
    member: &ClusterMemberView,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<StateDerivedConnectionTargetDto, CliError> {
    Ok(StateDerivedConnectionTargetDto {
        member_id: member_id.to_string(),
        conninfo: build_connection_conninfo(member, tls, emit_tls)?,
    })
}

fn build_connection_conninfo(
    member: &ClusterMemberView,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<PgConnInfo, CliError> {
    let tls = build_connection_tls(tls, emit_tls)?;
    Ok(PgConnInfo {
        endpoint: member.postgres_target().clone(),
        hostaddr: None,
        user: "postgres".to_string(),
        dbname: "postgres".to_string(),
        application_name: None,
        connect_timeout_s: None,
        options: None,
        tls,
    })
}

fn build_connection_tls(tls: &CliTlsConfig, emit_tls: bool) -> Result<PgClientTls, CliError> {
    /* one caller */
}
```
