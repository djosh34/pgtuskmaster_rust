use serde::{Deserialize, Serialize};

use crate::{
    api::NodeState,
    cli::{
        args::ConnectionOptions,
        client::CliTlsConfig,
        config::OperatorContext,
        error::CliError,
        output,
        status::{
            authority_primary_member, fetch_seed_state, member_is_ready_replica, ClusterWarning,
        },
    },
    dcs::DcsMemberView,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionCommandKind {
    Primary,
    Replicas,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionTarget {
    pub member_id: String,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub dsn: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionView {
    pub cluster_name: String,
    pub scope: String,
    pub kind: ConnectionCommandKind,
    pub tls: bool,
    pub discovered_member_count: usize,
    pub warnings: Vec<ClusterWarning>,
    pub targets: Vec<ConnectionTarget>,
}

pub(crate) async fn run_primary(
    context: &OperatorContext,
    options: ConnectionOptions,
) -> Result<String, CliError> {
    let (state, _queried_via) = fetch_seed_state(context).await?;
    let view = resolve_primary_view(
        &state,
        &context.postgres_client_tls,
        options.tls,
        context.primary_target.as_ref(),
    )?;
    output::render_connection_view(&view, options.json)
}

pub(crate) async fn run_replicas(
    context: &OperatorContext,
    options: ConnectionOptions,
) -> Result<String, CliError> {
    let (state, _queried_via) = fetch_seed_state(context).await?;
    let view = resolve_replicas_view(&state, &context.postgres_client_tls, options.tls)?;
    output::render_connection_view(&view, options.json)
}

fn resolve_primary_view(
    state: &NodeState,
    tls: &CliTlsConfig,
    emit_tls: bool,
    primary_target: Option<&crate::config::PgtmPrimaryTargetConfig>,
) -> Result<ConnectionView, CliError> {
    let primary_id = authority_primary_member(state).ok_or_else(|| {
        CliError::Resolution(
            "seed state does not currently expose an authoritative primary".to_string(),
        )
    })?;
    let member = state
        .dcs
        .members
        .get(&crate::state::MemberId(primary_id.clone()))
        .ok_or_else(|| {
            CliError::Resolution(format!(
                "authoritative primary `{primary_id}` is not present in the DCS member slots"
            ))
        })?;

    Ok(build_connection_view(
        state,
        emit_tls,
        ConnectionCommandKind::Primary,
        vec![build_primary_connection_target(
            member,
            tls,
            emit_tls,
            primary_target,
        )?],
    ))
}

fn resolve_replicas_view(
    state: &NodeState,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<ConnectionView, CliError> {
    let targets = state
        .dcs
        .members
        .values()
        .filter(|member| member_is_ready_replica(member))
        .map(|member| build_connection_target(member, tls, emit_tls))
        .collect::<Result<Vec<_>, _>>()?;

    if targets.is_empty() {
        return Err(CliError::Resolution(
            "seed state does not currently expose any ready replica members".to_string(),
        ));
    }

    Ok(build_connection_view(
        state,
        emit_tls,
        ConnectionCommandKind::Replicas,
        targets,
    ))
}

fn build_connection_view(
    state: &NodeState,
    emit_tls: bool,
    kind: ConnectionCommandKind,
    targets: Vec<ConnectionTarget>,
) -> ConnectionView {
    ConnectionView {
        cluster_name: state.cluster_name.clone(),
        scope: state.scope.clone(),
        kind,
        tls: emit_tls,
        discovered_member_count: state.dcs.members.len(),
        warnings: Vec::new(),
        targets,
    }
}

fn build_connection_target(
    member: &DcsMemberView,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<ConnectionTarget, CliError> {
    let postgres_host = member.routing.postgres.host.trim();
    if postgres_host.is_empty() || member.routing.postgres.port == 0 {
        return Err(CliError::Resolution(format!(
            "member {} does not advertise PostgreSQL host/port",
            member.member_id.0
        )));
    }

    let dsn = render_connection_dsn(postgres_host, member.routing.postgres.port, tls, emit_tls)?;
    Ok(ConnectionTarget {
        member_id: member.member_id.0.clone(),
        postgres_host: postgres_host.to_string(),
        postgres_port: member.routing.postgres.port,
        dsn,
    })
}

fn build_primary_connection_target(
    member: &DcsMemberView,
    tls: &CliTlsConfig,
    emit_tls: bool,
    override_target: Option<&crate::config::PgtmPrimaryTargetConfig>,
) -> Result<ConnectionTarget, CliError> {
    let resolved_host = override_target
        .map(|target| target.host.trim())
        .unwrap_or_else(|| member.routing.postgres.host.trim());
    let resolved_port = override_target
        .and_then(|target| target.port)
        .unwrap_or(member.routing.postgres.port);

    if resolved_host.is_empty() || resolved_port == 0 {
        return Err(CliError::Resolution(format!(
            "member {} does not advertise PostgreSQL host/port",
            member.member_id.0
        )));
    }

    let dsn = render_connection_dsn(resolved_host, resolved_port, tls, emit_tls)?;
    Ok(ConnectionTarget {
        member_id: member.member_id.0.clone(),
        postgres_host: resolved_host.to_string(),
        postgres_port: resolved_port,
        dsn,
    })
}

fn render_connection_dsn(
    postgres_host: &str,
    postgres_port: u16,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<String, CliError> {
    let mut fields = vec![
        ("host", postgres_host.to_string()),
        ("port", postgres_port.to_string()),
        ("user", "postgres".to_string()),
        ("dbname", "postgres".to_string()),
    ];
    if emit_tls {
        fields.push(("sslmode", "verify-full".to_string()));
        maybe_push_tls_path_field(
            &mut fields,
            "sslrootcert",
            "pgtm postgres client CA certificate",
            tls.ca_cert_pem.as_ref(),
            tls.ca_cert_path.as_ref(),
        )?;
        maybe_push_tls_path_field(
            &mut fields,
            "sslcert",
            "pgtm postgres client certificate",
            tls.client_cert_pem.as_ref(),
            tls.client_cert_path.as_ref(),
        )?;
        maybe_push_tls_path_field(
            &mut fields,
            "sslkey",
            "pgtm postgres client key",
            tls.client_key_pem.as_ref(),
            tls.client_key_path.as_ref(),
        )?;
    }

    Ok(fields
        .iter()
        .map(|(key, value)| format!("{key}={}", escape_libpq_value(value.as_str())))
        .collect::<Vec<_>>()
        .join(" "))
}

fn maybe_push_tls_path_field(
    fields: &mut Vec<(&'static str, String)>,
    dsn_key: &'static str,
    field_label: &'static str,
    pem: Option<&Vec<u8>>,
    path: Option<&std::path::PathBuf>,
) -> Result<(), CliError> {
    match (pem, path) {
        (Some(_), Some(path)) | (None, Some(path)) => {
            fields.push((dsn_key, path.to_string_lossy().into_owned()));
            Ok(())
        }
        (Some(_), None) => Err(CliError::Resolution(format!(
            "`--tls` cannot render {field_label} because the effective config is not path-backed"
        ))),
        (None, None) => Ok(()),
    }
}

fn escape_libpq_value(value: &str) -> String {
    let requires_quotes = value.is_empty()
        || value
            .chars()
            .any(|ch| ch.is_whitespace() || ch == '\'' || ch == '\\');
    if !requires_quotes {
        return value.to_string();
    }

    let escaped = value.chars().fold(String::new(), |mut acc, ch| {
        match ch {
            '\'' | '\\' => {
                acc.push('\\');
                acc.push(ch);
            }
            _ => acc.push(ch),
        }
        acc
    });
    format!("'{escaped}'")
}
