use std::env;

pub const AUTHOR: &str = "Bohdan Sokolovskyi (gaussfff)";
pub const REPOSITORY: &str = "https://github.com/gaussfff/anki-multitool";
pub const DEFAULT_HOST: &str = "localhost";
pub const DEFAULT_PORT: u16 = 8765;

pub const ASCII_ART: [&str; 6] = [
    " █████╗ ███╗   ██╗██╗  ██╗██╗    ███╗   ███╗██╗   ██╗██╗  ████████╗██╗████████╗ ██████╗  ██████╗ ██╗     ",
    "██╔══██╗████╗  ██║██║ ██╔╝██║    ████╗ ████║██║   ██║██║  ╚══██╔══╝██║╚══██╔══╝██╔═══██╗██╔═══██╗██║     ",
    "███████║██╔██╗ ██║█████╔╝ ██║    ██╔████╔██║██║   ██║██║     ██║   ██║   ██║   ██║   ██║██║   ██║██║     ",
    "██╔══██║██║╚██╗██║██╔═██╗ ██║    ██║╚██╔╝██║██║   ██║██║     ██║   ██║   ██║   ██║   ██║██║   ██║██║     ",
    "██║  ██║██║ ╚████║██║  ██╗██║    ██║ ╚═╝ ██║╚██████╔╝███████╗██║   ██║   ██║   ╚██████╔╝╚█████╔╝████████╗",
    "╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚═╝    ╚═╝     ╚═╝ ╚═════╝ ╚══════╝╚═╝   ╚═╝   ╚═╝    ╚═════╝  ╚════╝ ╚══════╝",
];

pub const APP_INFO: [&str; 6] = [
    "Anki-Multitool (Anki-MTool) - just useful multitool for Anki users. What it can do:",
    "  - Export decks to JSON and Markdown files 🃏 -> 📄",
    "  - Import decks from JSON and Markdown files 📄 -> 🃏",
    "  - List all decks in your Anki collection 📋",
    "  - Create a new deck ➕🃏",
    "  - To be continued... ⏩",
];

pub fn get_host() -> String {
    env::var("ANKI_MULTITOOL_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string())
}

pub fn get_port() -> u16 {
    env::var("ANKI_MULTITOOL_PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}
