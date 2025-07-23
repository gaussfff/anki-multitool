use std::env;

pub const PROGRAM_NAME: &str = "Anki-Multitool (Anki-MTool)";
pub const DEFAULT_HOST: &str = "localhost";
pub const DEFAULT_PORT: u16 = 8765;

pub fn get_host() -> String {
    env::var("ANKI_MULTITOOL_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string())
}

pub fn get_port() -> u16 {
    env::var("ANKI_MULTITOOL_PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}
