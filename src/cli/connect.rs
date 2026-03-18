use crate::{
    api::NodeState,
    cli::{
        args::ConnectionOptions,
        client::CliTlsConfig,
        config::OperatorContext,
        error::CliError,
        output,
        status::{
            authority_primary_member, build_state_projection, fetch_seed_state,
            member_is_ready_replica,
        },
    },
    command::{
        CommandOutputDto, StateDerivedConnectionCommandDto, StateDerivedConnectionCommandKind,
        StateDerivedConnectionTargetDto, StateQueryOriginDto,
    },
    dcs::ClusterMemberView,
    pginfo::{
        conninfo::{PgClientTls, PgSslMode},
        state::PgConnInfo,
    },
};

pub(crate) async fn run_primary(
    context: &OperatorContext,
    options: ConnectionOptions,
) -> Result<String, CliError> {
    let (state, queried_via) = fetch_seed_state(context).await?;
    let view = resolve_primary_view(
        &state,
        queried_via,
        &context.postgres_client_tls,
        options.tls,
    )?;
    output::render_command_output(&CommandOutputDto::Primary { output: view }, options.json)
}

pub(crate) async fn run_replicas(
    context: &OperatorContext,
    options: ConnectionOptions,
) -> Result<String, CliError> {
    let (state, queried_via) = fetch_seed_state(context).await?;
    let view = resolve_replicas_view(
        &state,
        queried_via,
        &context.postgres_client_tls,
        options.tls,
    )?;
    output::render_command_output(&CommandOutputDto::Replicas { output: view }, options.json)
}

fn resolve_primary_view(
    state: &NodeState,
    queried_via: StateQueryOriginDto,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<StateDerivedConnectionCommandDto, CliError> {
    let primary_id = authority_primary_member(state).ok_or_else(|| {
        CliError::Resolution(
            "seed state does not currently expose an authoritative primary".to_string(),
        )
    })?;
    let member = state
        .dcs
        .member(&crate::state::MemberId(primary_id.clone()))
        .ok_or_else(|| {
            CliError::Resolution(format!(
                "authoritative primary `{primary_id}` is not present in the DCS member slots"
            ))
        })?;

    Ok(build_connection_view(
        state,
        queried_via,
        StateDerivedConnectionCommandKind::Primary,
        vec![build_connection_target(primary_id.as_str(), member, tls, emit_tls)?],
    ))
}

fn resolve_replicas_view(
    state: &NodeState,
    queried_via: StateQueryOriginDto,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<StateDerivedConnectionCommandDto, CliError> {
    let targets = state
        .dcs
        .members()
        .filter(|(_member_id, member)| member_is_ready_replica(member))
        .map(|(member_id, member)| build_connection_target(member_id.0.as_str(), member, tls, emit_tls))
        .collect::<Result<Vec<_>, _>>()?;

    if targets.is_empty() {
        return Err(CliError::Resolution(
            "seed state does not currently expose any ready replica members".to_string(),
        ));
    }

    Ok(build_connection_view(
        state,
        queried_via,
        StateDerivedConnectionCommandKind::Replicas,
        targets,
    ))
}

fn build_connection_view(
    state: &NodeState,
    queried_via: StateQueryOriginDto,
    kind: StateDerivedConnectionCommandKind,
    targets: Vec<StateDerivedConnectionTargetDto>,
) -> StateDerivedConnectionCommandDto {
    StateDerivedConnectionCommandDto {
        projection: build_state_projection(state, queried_via, false),
        kind,
        targets,
    }
}

fn build_connection_target(
    member_id: &str,
    member: &ClusterMemberView,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<StateDerivedConnectionTargetDto, CliError> {
    let postgres_host = member.postgres_target().host().trim();
    let postgres_port = member.postgres_target().port();
    if postgres_host.is_empty() || postgres_port == 0 {
        return Err(CliError::Resolution(
            "member does not advertise PostgreSQL host/port".to_string(),
        ));
    }

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
        user: "postgres".to_string(),
        dbname: "postgres".to_string(),
        application_name: None,
        connect_timeout_s: None,
        ssl_mode: tls.mode,
        ssl_root_cert: tls.root_cert.clone(),
        options: None,
        tls,
    })
}

fn build_connection_tls(tls: &CliTlsConfig, emit_tls: bool) -> Result<PgClientTls, CliError> {
    if !emit_tls {
        return Ok(PgClientTls {
            mode: PgSslMode::Disable,
            root_cert: None,
            client_cert: None,
            client_key: None,
        });
    }

    Ok(PgClientTls {
        mode: PgSslMode::VerifyFull,
        root_cert: require_path_backed_tls_field(
            "pgtm postgres client CA certificate",
            tls.ca_cert_pem.as_ref(),
            tls.ca_cert_path.clone(),
        )?,
        client_cert: require_path_backed_tls_field(
            "pgtm postgres client certificate",
            tls.client_cert_pem.as_ref(),
            tls.client_cert_path.clone(),
        )?,
        client_key: require_path_backed_tls_field(
            "pgtm postgres client key",
            tls.client_key_pem.as_ref(),
            tls.client_key_path.clone(),
        )?,
    })
}

fn require_path_backed_tls_field(
    field_label: &'static str,
    pem: Option<&Vec<u8>>,
    path: Option<std::path::PathBuf>,
) -> Result<Option<std::path::PathBuf>, CliError> {
    match (pem, path) {
        (Some(_), Some(path)) | (None, Some(path)) => Ok(Some(path)),
        (Some(_), None) => Err(CliError::Resolution(format!(
            "`--tls` cannot render {field_label} because the effective config is not path-backed"
        ))),
        (None, None) => Ok(None),
    }
}
