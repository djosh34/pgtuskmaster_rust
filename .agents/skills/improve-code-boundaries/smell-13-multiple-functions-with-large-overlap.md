# Functions with wrong overlap

Many functions have too large roles.
This causes them to be unreusable.
Refactor by splitting up into parts (making use of the output being a new type with strong rust guarantees),
then reducing code by using the one shared abstraction.


Example:

This is wrong cuz:

- why is config setup in manage roles? Why not one setup config func?
- why is manage role both doing config, connection AND creating sql
- why is manage role doing the job of any sql connection? why not one shared sql connection thing?


```rust
pub(crate) async fn reconcile_managed_roles_v2(
    cfg: &RuntimeConfigV2,
) -> Result<(), RoleProvisionError> {
    let mut config = tokio_postgres::Config::new();
    config.host_path(cfg.postgres.socket_dir.as_path());
    config.port(cfg.postgres.listen_port);
    config.user(cfg.postgres.superuser.username.as_str());
    config.dbname(cfg.postgres.local_database.as_str());
    config.connect_timeout(cfg.postgres.connect_timeout);
    config.password(cfg.postgres.superuser.password.as_str());

    let (client, connection) = config
        .connect(NoTls)
        .await
        .map_err(|err| RoleProvisionError::Connect(err.to_string()))?;
    let connection_task = tokio::spawn(connection);

    let provision_sql = render_managed_role_reconciliation_sql_v2(cfg)?;
    client
        .batch_execute(provision_sql.as_str())
        .await
        .map_err(|err| RoleProvisionError::BatchExecute(err.to_string()))?;
    drop(client);

    let connection_result = connection_task
        .await
        .map_err(|err| RoleProvisionError::ConnectionJoin(err.to_string()))?;
    connection_result.map_err(|err| RoleProvisionError::Connection(err.to_string()))
}

```