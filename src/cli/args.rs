use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
}

#[derive(Clone, Debug, Parser)]
#[command(name = "pgtuskmasterctl")]
#[command(about = "HA admin CLI for PGTuskMaster API")]
pub struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:8008")]
    pub base_url: String,
    #[arg(long, env = "PGTUSKMASTER_READ_TOKEN")]
    pub read_token: Option<String>,
    #[arg(long, env = "PGTUSKMASTER_ADMIN_TOKEN")]
    pub admin_token: Option<String>,
    #[arg(long, default_value_t = 5_000)]
    pub timeout_ms: u64,
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub output: OutputFormat,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    Ha(HaArgs),
}

#[derive(Clone, Debug, Args)]
pub struct HaArgs {
    #[command(subcommand)]
    pub command: HaCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum HaCommand {
    State,
    Leader(LeaderArgs),
    Switchover(SwitchoverArgs),
}

#[derive(Clone, Debug, Args)]
pub struct LeaderArgs {
    #[command(subcommand)]
    pub command: LeaderCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum LeaderCommand {
    Set(SetLeaderArgs),
    Clear,
}

#[derive(Clone, Debug, Args)]
pub struct SetLeaderArgs {
    #[arg(long)]
    pub member_id: String,
}

#[derive(Clone, Debug, Args)]
pub struct SwitchoverArgs {
    #[command(subcommand)]
    pub command: SwitchoverCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SwitchoverCommand {
    Clear,
    Request(RequestSwitchoverArgs),
}

#[derive(Clone, Debug, Args)]
pub struct RequestSwitchoverArgs {
    #[arg(long)]
    pub requested_by: String,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::args::{
        Cli, Command, HaCommand, LeaderCommand, OutputFormat, SwitchoverCommand,
    };

    #[test]
    fn parse_ha_state_with_defaults() -> Result<(), String> {
        let cli = Cli::try_parse_from(["pgtuskmasterctl", "ha", "state"])
            .map_err(|err| format!("parse should succeed: {err}"))?;

        assert_eq!(cli.base_url, "http://127.0.0.1:8008");
        assert_eq!(cli.timeout_ms, 5_000);
        assert_eq!(cli.output, OutputFormat::Json);

        match cli.command {
            Command::Ha(ha) => match ha.command {
                HaCommand::State => Ok(()),
                _ => Err("expected ha state command".to_string()),
            },
        }
    }

    #[test]
    fn parse_requires_member_id_for_leader_set() {
        let parsed = Cli::try_parse_from(["pgtuskmasterctl", "ha", "leader", "set"]);
        assert!(parsed.is_err(), "member-id is required");
    }

    #[test]
    fn parse_requires_requested_by_for_switchover_request() {
        let parsed = Cli::try_parse_from(["pgtuskmasterctl", "ha", "switchover", "request"]);
        assert!(parsed.is_err(), "requested-by is required");
    }

    #[test]
    fn parse_full_write_commands() -> Result<(), String> {
        let cli = Cli::try_parse_from([
            "pgtuskmasterctl",
            "--base-url",
            "https://cluster.example",
            "--timeout-ms",
            "1234",
            "--output",
            "text",
            "ha",
            "leader",
            "set",
            "--member-id",
            "node-a",
        ])
        .map_err(|err| format!("parse should succeed: {err}"))?;

        assert_eq!(cli.base_url, "https://cluster.example");
        assert_eq!(cli.timeout_ms, 1234);
        assert_eq!(cli.output, OutputFormat::Text);

        match cli.command {
            Command::Ha(ha) => match ha.command {
                HaCommand::Leader(leader) => match leader.command {
                    LeaderCommand::Set(set) => {
                        assert_eq!(set.member_id, "node-a");
                        Ok(())
                    }
                    _ => Err("expected leader set".to_string()),
                },
                _ => Err("expected leader command".to_string()),
            },
        }
    }

    #[test]
    fn parse_switchover_request() -> Result<(), String> {
        let cli = Cli::try_parse_from([
            "pgtuskmasterctl",
            "ha",
            "switchover",
            "request",
            "--requested-by",
            "node-b",
        ])
        .map_err(|err| format!("parse should succeed: {err}"))?;

        match cli.command {
            Command::Ha(ha) => match ha.command {
                HaCommand::Switchover(switchover) => match switchover.command {
                    SwitchoverCommand::Request(request) => {
                        assert_eq!(request.requested_by, "node-b");
                        Ok(())
                    }
                    _ => Err("expected switchover request".to_string()),
                },
                _ => Err("expected switchover command".to_string()),
            },
        }
    }

    #[test]
    fn parse_env_token_fallbacks() -> Result<(), String> {
        let read_var = "PGTUSKMASTER_READ_TOKEN";
        let admin_var = "PGTUSKMASTER_ADMIN_TOKEN";

        std::env::set_var(read_var, "reader");
        std::env::set_var(admin_var, "admin");

        let parsed = Cli::try_parse_from(["pgtuskmasterctl", "ha", "state"])
            .map_err(|err| format!("parse should succeed: {err}"));

        std::env::remove_var(read_var);
        std::env::remove_var(admin_var);

        let cli = parsed?;
        assert_eq!(cli.read_token.as_deref(), Some("reader"));
        assert_eq!(cli.admin_token.as_deref(), Some("admin"));
        Ok(())
    }
}
