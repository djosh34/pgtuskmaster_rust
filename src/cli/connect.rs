use crate::{
    cli::{
        args::ConnectionOptions, client::CliTlsConfig, config::OperatorContext, error::CliError,
        status::fetch_seed_state,
    },
    command::{
        authoritative_primary_member, CommandOutputDto, StateDerivedConnectionCommandDto,
        StateDerivedConnectionTargetDto, StateProjectionDto,
    },
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
    let primary_id = authoritative_primary_member(&state).ok_or_else(|| {
        CliError::Resolution(
            "seed state does not currently expose an authoritative primary".to_string(),
        )
    })?;
    let member = state.dcs.member(primary_id).ok_or_else(|| {
        CliError::Resolution(format!(
            "authoritative primary `{}` is not present in the DCS member slots",
            primary_id.as_str()
        ))
    })?;
    let postgres_target = member.postgres_target();
    let postgres_host = postgres_target.host().trim();
    let postgres_port = postgres_target.port();
    if postgres_host.is_empty() || postgres_port == 0 {
        return Err(CliError::Resolution(
            "member does not advertise PostgreSQL host/port".to_string(),
        ));
    }
    let view = StateDerivedConnectionCommandDto {
        projection: StateProjectionDto::from_seed_state(&state, queried_via, false),
        targets: vec![StateDerivedConnectionTargetDto {
            member_id: primary_id.0.clone(),
            conninfo: PgConnInfo {
                endpoint: postgres_target.clone(),
                hostaddr: None,
                user: "postgres".to_string(),
                dbname: "postgres".to_string(),
                application_name: None,
                connect_timeout_s: None,
                options: None,
                tls: build_connection_tls(&context.postgres_client_tls, options.tls)?,
            },
        }],
    };
    CommandOutputDto::Primary { output: view }.render(options.json)
}

pub(crate) async fn run_replicas(
    context: &OperatorContext,
    options: ConnectionOptions,
) -> Result<String, CliError> {
    let (state, queried_via) = fetch_seed_state(context).await?;
    let connection_tls = build_connection_tls(&context.postgres_client_tls, options.tls)?;
    let targets = state
        .dcs
        .members()
        .filter(|(_member_id, member)| member.postgres().is_ready_replica())
        .map(|(member_id, member)| {
            let postgres_target = member.postgres_target();
            let postgres_host = postgres_target.host().trim();
            let postgres_port = postgres_target.port();
            if postgres_host.is_empty() || postgres_port == 0 {
                return Err(CliError::Resolution(
                    "member does not advertise PostgreSQL host/port".to_string(),
                ));
            }

            Ok(StateDerivedConnectionTargetDto {
                member_id: member_id.0.clone(),
                conninfo: PgConnInfo {
                    endpoint: postgres_target.clone(),
                    hostaddr: None,
                    user: "postgres".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: None,
                    options: None,
                    tls: connection_tls.clone(),
                },
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    if targets.is_empty() {
        return Err(CliError::Resolution(
            "seed state does not currently expose any ready replica members".to_string(),
        ));
    }

    let view = StateDerivedConnectionCommandDto {
        projection: StateProjectionDto::from_seed_state(&state, queried_via, false),
        targets,
    };
    CommandOutputDto::Replicas { output: view }.render(options.json)
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
