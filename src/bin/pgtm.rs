use std::process::ExitCode;

use clap::Parser;
use pgtuskmaster_rust::cli::args::Cli;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match pgtuskmaster_rust::cli::run(cli).await {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            err.exit_code()
        }
    }
}
