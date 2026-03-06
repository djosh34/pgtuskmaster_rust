use std::{path::PathBuf, process::ExitCode};

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "pgtuskmaster")]
#[command(about = "Run a pgtuskmaster node (or act as a Postgres archive/restore helper)")]
struct Cli {
    /// Path to runtime config file (node mode)
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Helper invoked by Postgres `archive_command` / `restore_command`
    Wal(WalArgs),
}

#[derive(Debug, Args)]
struct WalArgs {
    /// PGDATA path for locating pgtuskmaster-managed helper config
    #[arg(long, value_name = "PATH")]
    pgdata: PathBuf,
    #[command(subcommand)]
    command: WalCommand,
}

#[derive(Debug, Subcommand)]
enum WalCommand {
    /// Implements `archive_command` via pgBackRest `archive-push`
    ArchivePush {
        /// Full path to the WAL file (`%p`)
        wal_path: String,
    },
    /// Implements `restore_command` via pgBackRest `archive-get`
    ArchiveGet {
        /// WAL segment name (`%f`)
        wal_segment: String,
        /// Destination path (`%p`)
        destination_path: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Wal(args)) => run_wal(args),
        None => run_node(cli),
    }
}

fn run_node(cli: Cli) -> ExitCode {
    let config = match cli.config.as_ref() {
        Some(path) => path,
        None => {
            eprintln!("missing required `--config <PATH>` (or specify a subcommand)");
            return ExitCode::from(2);
        }
    };

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| err.to_string());
    let runtime = match runtime {
        Ok(value) => value,
        Err(err) => {
            eprintln!("failed to build tokio runtime: {err}");
            return ExitCode::from(1);
        }
    };

    let result = runtime.block_on(pgtuskmaster_rust::runtime::run_node_from_config_path(
        config.as_path(),
    ));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn run_wal(args: WalArgs) -> ExitCode {
    let pgdata = args.pgdata.as_path();
    let kind = match args.command {
        WalCommand::ArchivePush { wal_path } => {
            pgtuskmaster_rust::wal_passthrough::WalPassthroughKind::ArchivePush { wal_path }
        }
        WalCommand::ArchiveGet {
            wal_segment,
            destination_path,
        } => pgtuskmaster_rust::wal_passthrough::WalPassthroughKind::ArchiveGet {
            wal_segment,
            destination_path,
        },
    };

    let exit = pgtuskmaster_rust::wal_passthrough::run(pgdata, kind);
    match exit {
        Ok(value) => ExitCode::from(value.code),
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}
