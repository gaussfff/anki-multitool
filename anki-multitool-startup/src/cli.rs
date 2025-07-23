use clap::{
    Parser, 
    Subcommand, 
    builder::{
        Styles,
        styling::AnsiColor,
    }
};

#[derive(Parser)]
#[command(disable_version_flag = true)]
#[command(disable_help_flag = true)]
#[command(color = clap::ColorChoice::Auto)]
#[command(styles = get_styles())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(name = "info", about = "📝 Show description about anki-multitool")]
    Info,
    #[command(name = "version", about = "🏷️ Show version of anki-multitool")]
    Version,
    #[command(name = "decklist", about = "📋 List all decks in your Anki collection")]
    Decklist,
    #[command(name = "newdeck", about = "➕🃏 Create a new deck in Anki")]
    Newdeck {
        #[arg(value_name = "DECK_NAME", help = "Name of deck")]
        deck: String 
    },
    #[command(name = "json2deck", about = "📄 -> 🃏 Import a deck from a JSON file into Anki, if deck exists, it will return error")]
    Json2deck {
        #[arg(value_name = "PATH", help = "Path to the JSON file")]
        path: String 
    },
    #[command(name = "deck2json", about = "🃏 -> 📄 Export a deck from Anki to a JSON file, if file exists, it will return error")]
    Deck2json {
        #[arg(value_name = "DECK_NAME", help = "Name of deck to export")]
        deck: String 
    },
    #[command(name = "md2deck", about = "📄 -> 🃏 Import a deck from a Markdown file into Anki")]
    Md2deck {
        #[arg(value_name = "PATH", help = "Import a deck from a Markdown file into Anki, if deck exists, it will return error")]
        path: String 
    },
    #[command(name = "deck2md", about = "🃏 -> 📄 Export a deck from Anki to a Markdown file, if file exists, it will return error")]
    Deck2md {
        #[arg(value_name = "DECK_NAME", help = "Name of deck to export")]
        deck: String 
    },
}

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default())
        .usage(AnsiColor::Green.on_default())
        .literal(AnsiColor::Blue.on_default().bold())
        .error(AnsiColor::Red.on_default())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Red.on_default())
}

#[cfg(test)]
mod tests {
    mod util {
        use super::Cli;
        use anyhow::Result;
        use clap::{Error, Parser};

        pub fn parse_args(args: &[&str]) -> Result<Cli, Error> {
            Cli::try_parse_from(args)
        }
    }

    use super::*;
    use util::parse_args;

    #[test]
    fn test_cli() {
        let mut cli =
            parse_args(&["anki-mtool", "info"]).expect("failed to parse CLI arguments");
        assert!(matches!(cli.command, Command::Info));
        
        cli = parse_args(&["anki-mtool", "version"]).expect("failed to parse CLI arguments");
        assert!(matches!(cli.command, Command::Version));

        cli = parse_args(&["anki-mtool", "decklist"]).expect("failed to parse CLI arguments");
        assert!(matches!(cli.command, Command::Decklist));

        cli = parse_args(&["anki-mtool", "newdeck", "test_deck"])
            .expect("failed to parse CLI arguments");
        assert!(matches!(cli.command, Command::Newdeck { deck } if deck == "test_deck"));

        cli = parse_args(&["anki-mtool", "json2deck", "path/to/file.json"])
            .expect("failed to parse CLI arguments");
        assert!(matches!(cli.command, Command::Json2deck { path } if path == "path/to/file.json"));

        cli = parse_args(&["anki-mtool", "deck2json", "test_deck"])
            .expect("failed to parse CLI arguments");
        assert!(matches!(cli.command, Command::Deck2json { deck } if deck == "test_deck"));

        cli = parse_args(&["anki-mtool", "md2deck", "path/to/file.md"])
            .expect("failed to parse CLI arguments");
        assert!(matches!(cli.command, Command::Md2deck { path } if path == "path/to/file.md"));

        cli = parse_args(&["anki-mtool", "deck2md", "test_deck"])
            .expect("failed to parse CLI arguments");
        assert!(matches!(cli.command, Command::Deck2md { deck } if deck == "test_deck"));
    }

    #[test]
    fn test_failed_cli() {
        let cli = parse_args(&["anki-mtool", "unknown_command"]);
        assert!(cli.is_err());
    }
}
