use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ply")]
#[command(about = "Rust chess toolkit for parsing and analysis")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Validate {
        file: PathBuf,
        /// Print per-game replay errors for invalid games
        #[arg(short, long)]
        verbose: bool,
    },
    Summarize {
        file: PathBuf,
    },
    Stats {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
    Fen {
        fen: String,
        #[arg(long)]
        legal_moves: bool,
    },
    Perft {
        #[arg(long)]
        fen: Option<String>,
        #[arg(long)]
        depth: u8,
        #[arg(long)]
        divide: bool,
    },
}
