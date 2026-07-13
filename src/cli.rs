use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Debug, Parser)]
#[command(name = "adr")]
#[command(about = "Manage architecture decision records", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize ADR management in the current repository
    Init {
        /// Path to the ADR directory, relative to the current working directory
        adr_directory: std::path::PathBuf,
    },
    /// Create a new ADR draft
    New {
        /// ADR title
        title: String,
    },
    /// Modify an existing ADR
    #[command(name = "mod")]
    Mod {
        /// ADR id
        id: u32,
        /// Accept ADR
        #[arg(short = 'a', long = "accept", default_value_t = false)]
        accept: bool,
        /// ID to supersede
        #[arg(short = 's', long = "supersede")]
        supersede: Option<u32>,
    },
    /// Regenerate ADR overview document
    Toc,
    /// Generate shell completion script
    Completion {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}
