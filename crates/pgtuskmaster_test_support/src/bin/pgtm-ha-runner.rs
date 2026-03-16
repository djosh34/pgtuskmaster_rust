use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "pgtm-ha-runner")]
#[command(about = "Run the HA scenario helper daemon inside the HA runner container")]
struct Cli {
    #[arg(long, value_name = "PATH")]
    contract_dir: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match pgtuskmaster_test_support::ha_runner::run_daemon(cli.contract_dir.as_path()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}
