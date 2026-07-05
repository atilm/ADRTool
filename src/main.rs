mod cli;
mod init;
mod new;
mod output;
mod resolver;
mod template;
mod toc;

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
        cli::Commands::New { title } => match new::run(&title) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                output.error(&err.to_string());
                ExitCode::from(1)
            }
        },
        cli::Commands::Toc => match toc::run() {
            Ok(result) => {
                for warning in result.warnings {
                    output.warning(warning.message.as_str());
                }
                ExitCode::SUCCESS
            }
            Err(err) => {
                output.error(&err.to_string());
                ExitCode::from(1)
            }
        },
        cli::Commands::Mod { .. } => {
            output.error("command not implemented yet");
            ExitCode::from(1)
        }
    }
}
