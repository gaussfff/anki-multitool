use anyhow::Result;
use colored::Colorize;

use crate::config::{APP_INFO, ASCII_ART, AUTHOR, REPOSITORY};

pub fn print_info(version: &str) {
    for line in ASCII_ART {
        println!("{}", line.bold().blue());
    }

    println!("{}{}\n", "by ".green(), AUTHOR.bold().red());

    println!("{}{}", "version: ".green(), version.bold().red());

    println!("{}{}\n", "repository: ".green(), REPOSITORY.bold().red());

    for line in APP_INFO {
        println!("{}", line.green());
    }
}

pub fn print_version(version: &str) {
    println!("{}{}", "v.".green(), version.bold().blue());
}

pub fn print_new_deck(res: Result<u64>, deck_name: &str) {
    match res {
        Ok(deck_id) => {
            println!(
                "{}{}{}{}{}",
                "deck '".green(),
                deck_name.bold().blue(),
                "' was created with id '".green(),
                deck_id.to_string().bold().blue(),
                "'".green()
            );
        }
        Err(e) => {
            println!(
                "{}{}{}",
                "error creating deck".red(),
                ": ".red(),
                e.to_string().bold().red()
            );
        }
    }
}

pub fn print_decklist(decks: Result<Vec<String>>) {
    match decks {
        Ok(decks) => {
            if decks.is_empty() {
                println!("{}", "no decks found".red());
            } else {
                println!("{}", "decks:".green());
                for deck in decks {
                    println!("{} {}", "-".green(), deck.bold().blue());
                }
            }
        }
        Err(e) => {
            println!(
                "{}{}",
                "error fetching deck list: ".red(),
                e.to_string().bold().red()
            );
        }
    }
}

pub fn print_deck2md(file: Result<String>, deck: &str) {
    match file {
        Ok(file) => {
            println!(
                "{}{}{}{}{}",
                "deck '".green(),
                deck.bold().blue(),
                "' was written to '".green(),
                file.bold().blue(),
                "' file".green()
            );
        }
        Err(e) => {
            println!(
                "{}{}",
                "error converting deck to markdown: ".red(),
                e.to_string().bold().red()
            );
        }
    }
}

pub fn print_deck2json(file: Result<String>, deck: &str) {
    match file {
        Ok(file) => {
            println!(
                "{}{}{}{}{}",
                "deck '".green(),
                deck.bold().blue(),
                "' was written to '".green(),
                file.bold().blue(),
                "' file".green()
            );
        }
        Err(e) => {
            println!(
                "{}{}",
                "error converting deck to json: ".red(),
                e.to_string().bold().red()
            );
        }
    }
}

pub fn print_md2deck(deck: Result<String>, path: &str) {
    match deck {
        Ok(deck) => {
            println!(
                "{}{}{}{}{}",
                "deck '".green(),
                deck.bold().blue(),
                "' was created from '".green(),
                path.bold().blue(),
                "' file".green()
            );
        }
        Err(e) => {
            println!(
                "{}{}",
                "error converting markdown to deck: ".red(),
                e.to_string().bold().red()
            );
        }
    }
}

pub fn print_json2deck(deck: Result<String>, path: &str) {
    match deck {
        Ok(deck) => {
            println!(
                "{}{}{}{}{}",
                "deck '".green(),
                deck.bold().blue(),
                "' was created from '".green(),
                path.bold().blue(),
                "' file".green()
            );
        }
        Err(e) => {
            println!(
                "{}{}",
                "error converting json to deck: ".red(),
                e.to_string().bold().red()
            );
        }
    }
}
