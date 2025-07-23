use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(disable_version_flag = true)]
#[command(disable_help_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Version,
    Decklist,
    Newdeck { deck: String },
    Json2deck { path: String },
    Deck2json { deck: String },
    Md2deck { path: String },
    Deck2md { deck: String },
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
            parse_args(&["anki-mtool", "version"]).expect("failed to parse CLI arguments");
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
