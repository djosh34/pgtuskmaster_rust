use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

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
    #[arg(long, global = true)]
    pub json: bool,
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,
    #[arg(long, global = true)]
    pub watch: bool,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    Status,
    Primary(ConnectionArgs),
    Replicas(ConnectionArgs),
    Switchover(SwitchoverArgs),
}

#[derive(Clone, Debug, Args, PartialEq, Eq)]
pub struct ConnectionArgs {
    #[arg(long)]
    pub tls: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct StatusOptions {
    pub json: bool,
    pub verbose: bool,
    pub watch: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ConnectionOptions {
    pub json: bool,
    pub tls: bool,
}

impl Cli {
    pub fn status_options(&self) -> StatusOptions {
        StatusOptions {
            json: self.json,
            verbose: self.verbose,
            watch: self.watch,
        }
    }

    pub fn connection_options(&self, args: &ConnectionArgs) -> ConnectionOptions {
        ConnectionOptions {
            json: self.json,
            tls: args.tls,
        }
    }
}
