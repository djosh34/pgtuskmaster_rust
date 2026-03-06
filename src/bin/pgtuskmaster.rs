use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "pgtuskmaster")]
#[command(about = "Run a pgtuskmaster node")]
struct Cli {
    /// Path to runtime config file
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    run_node(cli)
}

fn run_node(cli: Cli) -> ExitCode {
    let config = match cli.config.as_ref() {
        Some(path) => path,
        None => {
            eprintln!("missing required `--config <PATH>`");
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
