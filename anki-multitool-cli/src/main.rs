mod cli;

use clap::Parser;
use cli::{
    Cli,
    Command
};

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Command::Version => {
            println!("Anki-multitool version: {}", env!("CARGO_PKG_VERSION"));
        }
    }
}
