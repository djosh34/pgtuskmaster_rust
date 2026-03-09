use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
}

#[derive(Clone, Debug, Parser)]
#[command(name = "pgtm")]
#[command(about = "HA admin CLI for PGTuskMaster API")]
pub struct Cli {
    #[arg(short = 'c', long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub base_url: Option<String>,
    #[arg(long)]
    pub read_token: Option<String>,
    #[arg(long)]
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
    Status,
    Switchover(SwitchoverArgs),
}

#[derive(Clone, Debug, Args)]
pub struct SwitchoverArgs {
    #[command(subcommand)]
    pub command: SwitchoverCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SwitchoverCommand {
    Clear,
    Request(SwitchoverRequestArgs),
}

#[derive(Clone, Debug, Args)]
pub struct SwitchoverRequestArgs {
    #[arg(long)]
    pub switchover_to: Option<String>,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::args::{Cli, Command, OutputFormat, SwitchoverCommand, SwitchoverRequestArgs};

    #[test]
    fn parse_status_with_defaults() -> Result<(), String> {
        let cli = Cli::try_parse_from(["pgtm", "status"])
            .map_err(|err| format!("parse should succeed: {err}"))?;

        assert_eq!(cli.config, None);
        assert_eq!(cli.base_url, None);
        assert_eq!(cli.timeout_ms, 5_000);
        assert_eq!(cli.output, OutputFormat::Json);

        match cli.command {
            Command::Status => Ok(()),
            _ => Err("expected status command".to_string()),
        }
    }

    #[test]
    fn parse_full_switchover_write_command() -> Result<(), String> {
        let cli = Cli::try_parse_from([
            "pgtm",
            "-c",
            "/tmp/pgtm.toml",
            "--base-url",
            "https://cluster.example",
            "--timeout-ms",
            "1234",
            "--output",
            "text",
            "switchover",
            "request",
        ])
        .map_err(|err| format!("parse should succeed: {err}"))?;

        assert_eq!(
            cli.config.as_deref(),
            Some(std::path::Path::new("/tmp/pgtm.toml"))
        );
        assert_eq!(cli.base_url.as_deref(), Some("https://cluster.example"));
        assert_eq!(cli.timeout_ms, 1234);
        assert_eq!(cli.output, OutputFormat::Text);

        match cli.command {
            Command::Switchover(switchover) => match switchover.command {
                SwitchoverCommand::Request(SwitchoverRequestArgs {
                    switchover_to: None,
                }) => Ok(()),
                _ => Err("expected switchover request".to_string()),
            },
            _ => Err("expected switchover command".to_string()),
        }
    }

    #[test]
    fn parse_switchover_request() -> Result<(), String> {
        let cli = Cli::try_parse_from(["pgtm", "switchover", "request"])
            .map_err(|err| format!("parse should succeed: {err}"))?;

        match cli.command {
            Command::Switchover(switchover) => match switchover.command {
                SwitchoverCommand::Request(SwitchoverRequestArgs {
                    switchover_to: None,
                }) => Ok(()),
                _ => Err("expected switchover request".to_string()),
            },
            _ => Err("expected switchover command".to_string()),
        }
    }

    #[test]
    fn parse_targeted_switchover_request() -> Result<(), String> {
        let cli =
            Cli::try_parse_from(["pgtm", "switchover", "request", "--switchover-to", "node-b"])
                .map_err(|err| format!("parse should succeed: {err}"))?;

        match cli.command {
            Command::Switchover(switchover) => match switchover.command {
                SwitchoverCommand::Request(SwitchoverRequestArgs {
                    switchover_to: Some(member_id),
                }) if member_id == "node-b" => Ok(()),
                _ => Err("expected targeted switchover request".to_string()),
            },
            _ => Err("expected switchover command".to_string()),
        }
    }

    #[test]
    fn parse_env_token_fallbacks() -> Result<(), String> {
        let cli = Cli::try_parse_from(["pgtm", "-c", "/tmp/pgtm.toml", "status"])
            .map_err(|err| format!("parse should succeed: {err}"))?;
        assert_eq!(
            cli.config.as_deref(),
            Some(std::path::Path::new("/tmp/pgtm.toml"))
        );
        assert_eq!(cli.base_url, None);
        assert_eq!(cli.read_token, None);
        assert_eq!(cli.admin_token, None);
        Ok(())
    }
}
