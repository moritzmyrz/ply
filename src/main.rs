mod cli;

use clap::Parser;

use crate::cli::commands::Cli;

fn main() {
    let cli = Cli::parse();
    if let Err(err) = crate::cli::run(cli) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
