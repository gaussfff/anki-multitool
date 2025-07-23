use anki_multitool_ds::http::{
    request::{ApiRequest, Note},
    response::ApiResponse,
};
use anyhow::{Result, anyhow};
use reqwest::{Client, Method};

#[derive(Debug, Clone)]
pub struct AnkiClient {
    client: Client,
    host: String,
    port: u16,
}

impl AnkiClient {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            client: Client::new(),
            host,
            port,
        }
    }

    pub async fn is_deck_exists(&self, deck: &str) -> Result<bool> {
        Ok(self
            .deck_names_req()
            .await?
            .into_result()?
            .into_names_res()
            .unwrap_or(Vec::new())
            .iter()
            .any(|d| d == deck))
    }

    pub async fn create_deck_req(&self, deck_name: &str) -> Result<ApiResponse> {
        if self.is_deck_exists(deck_name).await? {
            return Err(anyhow!("deck '{deck_name}' already exists"));
        }

        self.post_request(ApiRequest::make_create_deck_req(deck_name))
            .await
    }

    pub async fn deck_names_req(&self) -> Result<ApiResponse> {
        self.post_request(ApiRequest::make_deck_names_req()).await
    }

    pub async fn notes_info_req(&self, deck: &str) -> Result<ApiResponse> {
        if !self.is_deck_exists(deck).await? {
            return Err(anyhow!("deck '{deck}' does not exist"));
        }

        self.post_request(ApiRequest::make_notes_info_req(deck))
            .await
    }

    pub async fn deck_names_and_ids_req(&self) -> Result<ApiResponse> {
        self.post_request(ApiRequest::make_deck_names_and_ids_req())
            .await
    }

    pub async fn add_note_req(&self, note: Note) -> Result<ApiResponse> {
        if !self.is_deck_exists(&note.deck).await? {
            return Err(anyhow!("deck '{}' does not exist", note.deck));
        }

        self.post_request(ApiRequest::make_add_note_req(note)).await
    }

    pub async fn get_request(&self, request: ApiRequest) -> Result<ApiResponse> {
        self.request(Method::GET, request).await
    }

    pub async fn post_request(&self, request: ApiRequest) -> Result<ApiResponse> {
        self.request(Method::POST, request).await
    }

    pub async fn request(&self, method: Method, request: ApiRequest) -> Result<ApiResponse> {
        self.client
            .request(method, format!("http://{}:{}", self.host, self.port))
            .json(&request)
            .send()
            .await?
            .json::<ApiResponse>()
            .await
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anki_multitool_test_util::{server::MockAnkiServer, with_mserver};

    const HOST: &str = "localhost";

    #[tokio::test]
    pub async fn test_decks() {
        use std::collections::HashMap;

        let port = 8766;

        with_mserver! {
            use_port port;

            let client = AnkiClient::new(HOST.to_string(), port);

            assert!(client.create_deck_req("Deck 1").await.is_ok());
            assert!(client.create_deck_req("Deck 2").await.is_ok());
            assert!(client.create_deck_req("Deck 3").await.is_ok());

            assert!(client.create_deck_req("Deck 3").await.is_err());

            assert_eq!(
                client
                    .deck_names_req()
                    .await
                    .unwrap()
                    .into_result()
                    .unwrap()
                    .into_names_res()
                    .unwrap(),
                vec![
                    "Deck 1".to_string(),
                    "Deck 2".to_string(),
                    "Deck 3".to_string()
                ]
            );

            assert_eq!(
                client
                    .deck_names_and_ids_req()
                    .await
                    .unwrap()
                    .into_result()
                    .unwrap()
                    .into_names_and_ids_res()
                    .unwrap(),
                HashMap::from([
                    ("Deck 1".to_string(), 0),
                    ("Deck 2".to_string(), 1),
                    ("Deck 3".to_string(), 2),
                ])
            );

            assert!(client.notes_info_req("Unknown deck").await.is_err());
        }
    }

    #[tokio::test]
    pub async fn test_notes() {
        use anki_multitool_ds::http::response::NotesInfoResponseData;

        let port = 8787;
        let _server = MockAnkiServer::new(HOST, port)
            .await
            .expect("failed to create mock server");
        let client = AnkiClient::new(HOST.to_string(), port);

        with_mserver! {
            use_port port;

            assert!(client.create_deck_req("Test Deck").await.is_ok());
            assert!(
                client
                    .add_note_req(Note::new(
                        "Test Deck".to_string(),
                        "Q1".to_string(),
                        "A1".to_string()
                    ))
                    .await
                    .is_ok()
            );
            assert!(
                client
                    .add_note_req(Note::new(
                        "Test Deck".to_string(),
                        "Q2".to_string(),
                        "A2".to_string()
                    ))
                    .await
                    .is_ok()
            );
            assert!(
                client
                    .add_note_req(Note::new(
                        "Test Deck".to_string(),
                        "Q3".to_string(),
                        "A3".to_string()
                    ))
                    .await
                    .is_ok()
            );

            assert!(
                client
                    .add_note_req(Note::new(
                        "Test Deck 1".to_string(),
                        "Q1".to_string(),
                        "A1".to_string()
                    ))
                    .await
                    .is_err()
            );

            let res = client.notes_info_req("Test Deck").await;
            assert!(res.is_ok());

            assert_eq!(
                res.unwrap()
                    .into_result()
                    .unwrap()
                    .into_notes_info_res()
                    .unwrap(),
                vec![
                    NotesInfoResponseData::new_simple("Q1", "A1").with_id(0),
                    NotesInfoResponseData::new_simple("Q2", "A2").with_id(1),
                    NotesInfoResponseData::new_simple("Q3", "A3").with_id(2),
                ]
            );
        }
    }
}
