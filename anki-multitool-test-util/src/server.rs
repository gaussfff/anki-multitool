use anyhow::Result;
use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{Arc, Mutex, atomic::AtomicUsize},
};
use wiremock::{
    Mock, MockServer, Request, Respond, ResponseTemplate,
    matchers::{body_json_schema, method, path},
};

use anki_multitool_ds::http::{
    request::{
        AddNoteParams, ApiMethod, ApiRequest, CreateDeckParams, Note, NotesInfoParams, Params,
    },
    response::{ApiResponse, NotesInfoResponseData},
};

type Decks = Arc<Mutex<HashMap<String, (u64, HashMap<u64, NotesInfoResponseData>)>>>;

#[macro_export]
macro_rules! with_mserver {
    (use_port $port:expr; $($body:stmt;)*) => {
        {
            let __mock_server = MockAnkiServer::new("localhost", $port).await.expect("failed to create mock server");
            $($body)*
        }
    }
}

pub struct MockAnkiServer {
    _state: State,
    _mock_server: MockServer,
}

impl MockAnkiServer {
    pub async fn new(host: &str, port: u16) -> Result<Self> {
        let mock_server = MockServer::builder()
            .listener(TcpListener::bind(format!("{host}:{port}"))?)
            .start()
            .await;
        let state = State::new();

        mock_server
            .register(
                Mock::given(method("POST"))
                    .and(path("/"))
                    .and(body_json_schema::<ApiRequest>)
                    .respond_with(Responder::new(state.clone())),
            )
            .await;

        Ok(Self {
            _state: state,
            _mock_server: mock_server,
        })
    }
}

#[derive(Clone)]
struct State {
    decks: Decks,
    deck_id_counter: Arc<AtomicUsize>,
    note_id_counter: Arc<AtomicUsize>,
}

impl State {
    fn new() -> Self {
        State {
            decks: Arc::new(Mutex::new(HashMap::new())),
            deck_id_counter: Arc::new(AtomicUsize::new(0)),
            note_id_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn next_deck_id(&self) -> u64 {
        self.deck_id_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst) as u64
    }

    fn next_note_id(&self) -> u64 {
        self.note_id_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst) as u64
    }

    fn deck_names(&self) -> Vec<String> {
        let mut res: Vec<String> = self.decks.lock().unwrap().keys().cloned().collect();
        res.sort();
        res
    }

    fn deck_names_and_ids(&self) -> HashMap<String, u64> {
        self.decks
            .lock()
            .unwrap()
            .iter()
            .map(|(name, (id, _))| (name.clone(), *id))
            .collect()
    }

    fn create_deck(&self, deck: String) -> u64 {
        if !self.decks.lock().unwrap().contains_key(&deck) {
            let id = self.next_deck_id();
            self.decks
                .lock()
                .unwrap()
                .insert(deck, (id, HashMap::new()));
            id
        } else {
            self.decks.lock().unwrap().get(&deck).unwrap().0
        }
    }

    fn add_note(&self, note: Note) -> Option<u64> {
        let deck_name = note.deck.clone();
        let note = NotesInfoResponseData::from(note);

        if self.decks.lock().unwrap().contains_key(deck_name.as_str()) {
            let note_id = self.next_note_id();
            self.decks
                .lock()
                .unwrap()
                .get_mut(deck_name.as_str())
                .unwrap()
                .1
                .insert(note_id, note.with_id(note_id));

            return Some(note_id);
        }

        None
    }

    fn notes_info(&self, deck_name: &str) -> Option<Vec<NotesInfoResponseData>> {
        self.decks.lock().unwrap().get(deck_name).map(|(_, notes)| {
            let mut res: Vec<NotesInfoResponseData> = notes.values().cloned().collect();
            res.sort();
            res
        })
    }
}

struct Responder {
    state: State,
}

impl Responder {
    pub fn new(state: State) -> Self {
        Responder { state }
    }
}

impl Respond for Responder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body = request.body.as_ref();
        let request_str = String::from_utf8_lossy(body);
        let ok_response = ResponseTemplate::new(200);
        let bad_response = ResponseTemplate::new(400)
            .set_body_json(ApiResponse::as_error("invalid request".to_string()));

        match serde_json::from_str::<ApiRequest>(&request_str) {
            Ok(request) => match request.action {
                ApiMethod::DeckNames => ok_response
                    .set_body_json(ApiResponse::with_names_ok_res(self.state.deck_names())),
                ApiMethod::DeckNamesAndIds => ok_response.set_body_json(
                    ApiResponse::with_names_and_ids_ok_res(self.state.deck_names_and_ids()),
                ),
                ApiMethod::CreateDeck => {
                    if let Some(Params::CreateDeck(CreateDeckParams { deck })) = request.params {
                        if deck.is_empty() {
                            return bad_response;
                        }

                        ok_response.set_body_json(ApiResponse::with_id_ok_res(
                            self.state.create_deck(deck),
                        ))
                    } else {
                        bad_response
                    }
                }
                ApiMethod::AddNote => {
                    if let Some(Params::AddNote(AddNoteParams { note })) = request.params {
                        let res = self.state.add_note(note);

                        if res.is_none() {
                            return bad_response;
                        }

                        ok_response.set_body_json(ApiResponse::with_id_ok_res(res.unwrap()))
                    } else {
                        bad_response
                    }
                }
                ApiMethod::NotesInfo => {
                    if let Some(Params::NotesInfo(NotesInfoParams { query: deck })) = request.params
                    {
                        let res = self.state.notes_info(deck.as_str());

                        if res.is_none() {
                            return bad_response;
                        }

                        ok_response.set_body_json(ApiResponse::with_notes_info_ok_res(res.unwrap()))
                    } else {
                        bad_response
                    }
                }
            },
            Err(_) => bad_response,
        }
    }
}
