use anyhow::Result;
use std::{io::Error, path::Path};
use tempfile::NamedTempFile;

use anki_multitool_request::client::AnkiClient;

pub async fn load_decks(host: &str, port: u16, decks: Vec<String>) -> Result<()> {
    let client = AnkiClient::new(host.to_string(), port);

    for deck in decks {
        client.create_deck_req(deck.as_str()).await?;
    }

    Ok(())
}

pub fn write_to_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    use std::{fs::File, io::Write};
    File::create(path)?.write_all(content.as_bytes())?;
    Ok(())
}

pub fn temp_json_file() -> Result<NamedTempFile, Error> {
    temp_file(".json")
}

pub fn temp_md_file() -> Result<NamedTempFile, Error> {
    temp_file(".md")
}

fn temp_file(ext: &str) -> Result<NamedTempFile, Error> {
    use tempfile::Builder;
    Builder::new().prefix("test_deck_").suffix(ext).tempfile()
}
