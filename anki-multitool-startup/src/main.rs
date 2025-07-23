mod cli;
mod config;
mod printer;

use clap::Parser;
use config::{get_host, get_port};

use anki_multitool_core::ToolController;

use cli::{Cli, Command};

#[tokio::main]
async fn main() {
    let controller = ToolController::new(get_host(), get_port());

    match Cli::parse().command {
        Command::Info => printer::print_info(controller.version()),
        Command::Version => printer::print_version(controller.version()),
        Command::Newdeck { ref deck } => {
            printer::print_new_deck(controller.new_deck(deck).await, deck)
        }
        Command::Decklist => printer::print_decklist(controller.deck_list().await),
        Command::Deck2md { ref deck } => {
            printer::print_deck2md(controller.convert_deck_to_md(deck).await, deck)
        }
        Command::Deck2json { ref deck } => {
            printer::print_deck2json(controller.convert_deck_to_json(deck).await, deck)
        }
        Command::Json2deck { ref path } => {
            printer::print_json2deck(controller.convert_json_to_deck(path).await, path)
        }
        Command::Md2deck { ref path } => {
            printer::print_md2deck(controller.convert_md_to_deck(path).await, path)
        }
    }
}
