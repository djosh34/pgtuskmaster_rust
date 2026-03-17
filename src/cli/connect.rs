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
        CommandOutputDto, LocalConnectionMaterialization, PathBackedClientTlsDto,
        RenderedConnectionCommandDto, StateDerivedConnectionCommandDto,
        StateDerivedConnectionCommandKind, StateDerivedConnectionTargetDto, StateQueryOriginDto,
    },
    dcs::ClusterMemberView,
};

pub(crate) async fn run_primary(
    context: &OperatorContext,
    options: ConnectionOptions,
) -> Result<String, CliError> {
    let (state, queried_via) = fetch_seed_state(context).await?;
    let view = resolve_primary_view(&state, queried_via, &context.postgres_client_tls, options.tls)?;
    output::render_command_output(&CommandOutputDto::Primary { output: view }, options.json)
}

pub(crate) async fn run_replicas(
    context: &OperatorContext,
    options: ConnectionOptions,
) -> Result<String, CliError> {
    let (state, queried_via) = fetch_seed_state(context).await?;
    let view =
        resolve_replicas_view(&state, queried_via, &context.postgres_client_tls, options.tls)?;
    output::render_command_output(&CommandOutputDto::Replicas { output: view }, options.json)
}

fn resolve_primary_view(
    state: &NodeState,
    queried_via: StateQueryOriginDto,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<RenderedConnectionCommandDto, CliError> {
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
        vec![build_connection_target(primary_id.as_str(), member)?],
        build_local_connection_material(tls, emit_tls)?,
    ))
}

fn resolve_replicas_view(
    state: &NodeState,
    queried_via: StateQueryOriginDto,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<RenderedConnectionCommandDto, CliError> {
    let targets = state
        .dcs
        .members()
        .filter(|(_member_id, member)| member_is_ready_replica(member))
        .map(|(member_id, member)| build_connection_target(member_id.0.as_str(), member))
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
        build_local_connection_material(tls, emit_tls)?,
    ))
}

fn build_connection_view(
    state: &NodeState,
    queried_via: StateQueryOriginDto,
    kind: StateDerivedConnectionCommandKind,
    targets: Vec<StateDerivedConnectionTargetDto>,
    local_connection: LocalConnectionMaterialization,
) -> RenderedConnectionCommandDto {
    RenderedConnectionCommandDto {
        state: StateDerivedConnectionCommandDto {
            projection: build_state_projection(state, queried_via, false),
            kind,
            targets,
        },
        local_connection,
    }
}

fn build_connection_target(
    member_id: &str,
    member: &ClusterMemberView,
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
        postgres_host: postgres_host.to_string(),
        postgres_port,
    })
}

fn build_local_connection_material(
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<LocalConnectionMaterialization, CliError> {
    if !emit_tls {
        return Ok(LocalConnectionMaterialization::Plaintext);
    }

    let paths = PathBackedClientTlsDto {
        ca_cert_path: require_path_backed_tls_field(
            "pgtm postgres client CA certificate",
            tls.ca_cert_pem.as_ref(),
            tls.ca_cert_path.clone(),
        )?,
        client_cert_path: require_path_backed_tls_field(
            "pgtm postgres client certificate",
            tls.client_cert_pem.as_ref(),
            tls.client_cert_path.clone(),
        )?,
        client_key_path: require_path_backed_tls_field(
            "pgtm postgres client key",
            tls.client_key_pem.as_ref(),
            tls.client_key_path.clone(),
        )?,
    };
    Ok(LocalConnectionMaterialization::Tls { paths })
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
