mod cli;
mod init;
mod output;
#[allow(dead_code)]
mod resolver;
mod template;

use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    let output = output::Output::new();

    match cli.command {
        cli::Commands::Init { adr_directory } => match init::run(&adr_directory) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                output.error(&err.to_string());
                ExitCode::from(1)
            }
        },
        cli::Commands::New { .. } | cli::Commands::Mod { .. } | cli::Commands::Toc => {
            output.error("command not implemented yet");
            ExitCode::from(1)
        }
    }
}
