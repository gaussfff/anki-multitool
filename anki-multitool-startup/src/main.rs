mod cli;
mod config;

use clap::Parser;
use cli::{Cli, Command};
use config::{PROGRAM_NAME, get_host, get_port};

use anki_multitool_core::ToolController;

#[tokio::main]
async fn main() {
    let controller = ToolController::new(get_host(), get_port());

    match Cli::parse().command {
        Command::Version => {
            println!("{PROGRAM_NAME}: {}", controller.get_version());
        }
        Command::Decklist => match controller.get_deck_names().await {
            Ok(decks) => {
                if decks.is_empty() {
                    println!("no decks found");
                } else {
                    println!("decks:");
                    for deck in decks {
                        println!("- {deck}");
                    }
                }
            }
            Err(e) => {
                println!("error fetching deck list: {e}");
            }
        },
        Command::Newdeck { ref deck } => match controller.new_deck(deck).await {
            Ok(deck_id) => {
                println!("deck {deck} was created with id {deck_id}");
            }
            Err(e) => {
                println!("error creating deck: {e}");
            }
        },
        Command::Deck2md { ref deck } => match controller.convert_deck_to_md(deck).await {
            Ok(file) => {
                println!("deck {deck} was written to {file}");
            }
            Err(e) => {
                println!("error converting deck to markdown: {e}");
            }
        },
        Command::Deck2json { ref deck } => match controller.convert_deck_to_json(deck).await {
            Ok(file) => {
                println!("deck {deck} was written to {file}");
            }
            Err(e) => {
                println!("error converting deck to json: {e}");
            }
        },
        Command::Json2deck { ref path } => match controller.convert_json_to_deck(path).await {
            Ok(deck) => {
                println!("deck {deck} was created from {path} file");
            }
            Err(e) => {
                println!("error converting json to deck: {e}");
            }
        },
        Command::Md2deck { ref path } => match controller.convert_md_to_deck(path).await {
            Ok(deck) => {
                println!("deck {deck} was created from {path} file");
            }
            Err(e) => {
                println!("error converting markdown to deck: {e}");
            }
        },
    }
}
