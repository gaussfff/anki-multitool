use clap::{
    Parser,
    Subcommand 
};

#[derive(Parser, Debug)]
#[command(disable_version_flag = true)]
#[command(disable_help_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Version,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    
    fn parse_args(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).expect("failed to parse CLI arguments")
    }

    #[test]
    fn cli_parsing_version_test() {
        let cli = parse_args(&["anki-mtool", "version"]);
        match cli.command {
            Command::Version => {}
            _ => panic!("expected Version command"),
        }
    }    
}
