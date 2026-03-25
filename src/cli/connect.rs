use crate::{
    api::authoritative_primary_member,
    cli::{
        args::ConnectionOptions, config::OperatorContext, error::CliError, status::fetch_seed_state,
    },
    command::CommandOutputDto,
    pginfo::{
        conninfo::{PgClientTls, PgSslMode},
        state::PgConnInfo,
    },
};

pub(crate) async fn run_primary(
    context: &OperatorContext,
    options: ConnectionOptions,
) -> Result<String, CliError> {
    let (state, _api_url) = fetch_seed_state(context).await?;
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
    let postgres_target = member.operator_or_cluster_postgres_target();
    let postgres_host = postgres_target.host().trim();
    let postgres_port = postgres_target.port();
    if postgres_host.is_empty() || postgres_port == 0 {
        return Err(CliError::Resolution(
            "member does not advertise PostgreSQL host/port".to_string(),
        ));
    }
    let targets = vec![PgConnInfo {
        route: postgres_target.clone(),
        user: "postgres".to_string(),
        dbname: "postgres".to_string(),
        application_name: None,
        connect_timeout_s: None,
        options: None,
        tls: build_connection_tls(context.postgres_client_tls.as_ref(), options.tls),
    }];
    CommandOutputDto::Primary { targets }.render(options.json)
}

pub(crate) async fn run_replicas(
    context: &OperatorContext,
    options: ConnectionOptions,
) -> Result<String, CliError> {
    let (state, _api_url) = fetch_seed_state(context).await?;
    let connection_tls = build_connection_tls(context.postgres_client_tls.as_ref(), options.tls);
    let targets = state
        .dcs
        .members()
        .filter(|(_member_id, member)| member.postgres().is_ready_replica())
        .map(|(_member_id, member)| {
            let postgres_target = member.operator_or_cluster_postgres_target();
            let postgres_host = postgres_target.host().trim();
            let postgres_port = postgres_target.port();
            if postgres_host.is_empty() || postgres_port == 0 {
                return Err(CliError::Resolution(
                    "member does not advertise PostgreSQL host/port".to_string(),
                ));
            }

            Ok(PgConnInfo {
                route: postgres_target.clone(),
                user: "postgres".to_string(),
                dbname: "postgres".to_string(),
                application_name: None,
                connect_timeout_s: None,
                options: None,
                tls: connection_tls.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    if targets.is_empty() {
        return Err(CliError::Resolution(
            "seed state does not currently expose any ready replica members".to_string(),
        ));
    }

    CommandOutputDto::Replicas { targets }.render(options.json)
}

fn build_connection_tls(tls: Option<&PgClientTls>, emit_tls: bool) -> PgClientTls {
    if !emit_tls {
        return PgClientTls {
            mode: PgSslMode::Disable,
            root_cert: None,
            client_cert: None,
            client_key: None,
        };
    }

    tls.cloned().unwrap_or(PgClientTls {
        mode: PgSslMode::VerifyFull,
        root_cert: None,
        client_cert: None,
        client_key: None,
    })
}
