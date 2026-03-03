use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "pgtuskmaster")]
#[command(about = "Run a pgtuskmaster node from a runtime config file")]
struct Args {
    #[arg(long, value_name = "PATH")]
    config: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args = Args::parse();
    match pgtuskmaster_rust::runtime::run_node_from_config_path(args.config.as_path()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}
