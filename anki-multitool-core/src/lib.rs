use anyhow::{Result, anyhow};
use std::path::Path;

use anki_multitool_convert::{
    json::{FromJsonDeck, ToJsonDeck},
    markdown::{FromMarkdownDeck, ToMarkdownDeck},
};
use anki_multitool_ds::{card::Card, http::request::Note};
use anki_multitool_request::client::AnkiClient;
use anki_multitool_util::file;

pub struct ToolController {
    pub client: AnkiClient,
}

impl ToolController {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            client: AnkiClient::new(host, port),
        }
    }

    pub fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    pub async fn deck_list(&self) -> Result<Vec<String>> {
        self.client
            .deck_names_req()
            .await?
            .into_result()?
            .into_names_res()
            .ok_or_else(|| anyhow!("no result in response"))
    }

    pub async fn new_deck(&self, deck: &str) -> Result<u64> {
        self.client
            .create_deck_req(deck)
            .await?
            .into_result()?
            .into_id_res()
            .ok_or_else(|| anyhow!("no result in response"))
    }

    pub async fn convert_deck_to_json(&self, deck: &str) -> Result<String> {
        ToJsonDeck::new(deck)
            .write(
                self.client
                    .notes_info_req(deck)
                    .await?
                    .into_result()?
                    .into_notes_info_res()
                    .ok_or_else(|| anyhow!("no result in response"))?
                    .into_iter()
                    .map(|note| Card {
                        front: note.fields.front.value,
                        back: note.fields.back.value,
                    }),
            )
            .await
    }

    pub async fn convert_deck_to_md(&self, deck: &str) -> Result<String> {
        ToMarkdownDeck::new(deck)
            .write(
                self.client
                    .notes_info_req(deck)
                    .await?
                    .into_result()?
                    .into_notes_info_res()
                    .ok_or_else(|| anyhow!("no result in response"))?
                    .into_iter()
                    .map(|note| Card {
                        front: note.fields.front.value,
                        back: note.fields.back.value,
                    }),
            )
            .await
    }

    pub async fn convert_json_to_deck<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        if !path.as_ref().exists() {
            return Err(anyhow!("file {} does not exist", path.as_ref().display()));
        }
        
        let deck_name = file::to_file_name(path.as_ref())?;
        self.new_deck(deck_name.as_str()).await?;

        FromJsonDeck::new(path)?
            .for_each(async |card| {
                self.client
                    .add_note_req(Note::new(deck_name.to_string(), card.front, card.back))
                    .await?
                    .into_result()
                    .map(|_| ())
            })
            .await?;

        Ok(deck_name)
    }

    pub async fn convert_md_to_deck<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        if !path.as_ref().exists() {
            return Err(anyhow!("file {} does not exist", path.as_ref().display()));
        }
        
        let deck_name = file::to_file_name(path.as_ref())?;
        self.new_deck(deck_name.as_str()).await?;

        FromMarkdownDeck::new(path)?
            .for_each(async |card| {
                self.client
                    .add_note_req(Note::new(deck_name.to_string(), card.front, card.back))
                    .await?
                    .into_result()
                    .map(|_| ())
            })
            .await?;

        Ok(deck_name)
    }
}
