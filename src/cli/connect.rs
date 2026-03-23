use crate::{
    api::authoritative_primary_member,
    cli::{
        args::ConnectionOptions, config::OperatorContext, error::CliError, status::fetch_seed_state,
    },
    command::{
        CommandOutputDto, StateDerivedConnectionCommandDto, StateDerivedConnectionTargetDto,
        StateProjectionDto,
    },
    config_v2::types::OperatorClientTlsConfig,
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
                tls: build_connection_tls(&context.postgres_client_tls, options.tls),
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
    let connection_tls = build_connection_tls(&context.postgres_client_tls, options.tls);
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

fn build_connection_tls(tls: &OperatorClientTlsConfig, emit_tls: bool) -> PgClientTls {
    if !emit_tls {
        return PgClientTls {
            mode: PgSslMode::Disable,
            root_cert: None,
            client_cert: None,
            client_key: None,
        };
    }

    PgClientTls {
        mode: PgSslMode::VerifyFull,
        root_cert: tls.ca_cert.clone(),
        client_cert: tls.identity.as_ref().map(|identity| identity.cert.clone()),
        client_key: tls.identity.as_ref().map(|identity| identity.key.clone()),
    }
}
