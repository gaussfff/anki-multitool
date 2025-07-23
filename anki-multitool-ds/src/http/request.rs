use anyhow::Result;
use serde::{Serialize, Serializer};

const API_VERSION: u16 = 6;
const DEFAULT_DECK_NAME: &str = "Default";
const DEFAULT_DUPLICATE_SCOPE: &str = "deck";
const DEFAULT_MODEL_NAME: &str = "Basic";

#[derive(Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "test", derive(serde::Deserialize, Debug))]
pub enum ApiMethod {
    #[serde(rename = "deckNames")]
    DeckNames,
    #[serde(rename = "deckNamesAndIds")]
    DeckNamesAndIds,
    #[serde(rename = "createDeck")]
    CreateDeck,
    #[serde(rename = "addNote")]
    AddNote,
    #[serde(rename = "notesInfo")]
    NotesInfo,
}

#[derive(Serialize)]
#[cfg_attr(
    feature = "test",
    derive(serde::Deserialize, Debug, Default, Eq, PartialEq)
)]
pub struct CreateDeckParams {
    pub deck: String,
}

#[derive(Serialize)]
#[cfg_attr(
    feature = "test",
    derive(serde::Deserialize, Debug, Default, Eq, PartialEq)
)]
pub struct AddNoteParams {
    pub note: Note,
}

#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serde::Deserialize, Debug, Eq, PartialEq))]
pub struct Note {
    #[serde(rename = "deckName")]
    pub deck: String,
    #[serde(rename = "modelName")]
    pub model: String,
    pub fields: InFields,
    pub options: Options,
    pub tags: Vec<String>,
}

impl Note {
    pub fn new(deck: String, front: String, back: String) -> Self {
        Self {
            deck,
            fields: InFields { front, back },
            ..Self::default()
        }
    }
}

impl Default for Note {
    fn default() -> Self {
        Self {
            deck: DEFAULT_DECK_NAME.to_string(),
            model: DEFAULT_MODEL_NAME.to_string(),
            fields: InFields::default(),
            options: Options::default(),
            tags: Vec::new(),
        }
    }
}

#[derive(Serialize, Default)]
#[cfg_attr(feature = "test", derive(serde::Deserialize, Debug, Eq, PartialEq))]
pub struct InFields {
    #[serde(rename = "Front")]
    pub front: String,
    #[serde(rename = "Back")]
    pub back: String,
}

#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serde::Deserialize, Debug, Eq, PartialEq))]
pub struct Options {
    #[serde(rename = "allowDuplicate")]
    pub allow_dups: bool,
    #[serde(rename = "duplicateScope")]
    pub dup_scope: String,
    #[serde(rename = "duplicateScopeOptions")]
    pub options: DuplicateScopeOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            allow_dups: false,
            dup_scope: DEFAULT_DUPLICATE_SCOPE.to_string(),
            options: DuplicateScopeOptions::default(),
        }
    }
}

#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serde::Deserialize, Debug, Eq, PartialEq))]
pub struct DuplicateScopeOptions {
    #[serde(rename = "deckName")]
    pub deck_name: String,
    #[serde(rename = "checkChildren")]
    pub check_children: bool,
    #[serde(rename = "checkModels")]
    pub check_models: bool,
}

impl Default for DuplicateScopeOptions {
    fn default() -> Self {
        Self {
            deck_name: DEFAULT_DECK_NAME.to_string(),
            check_children: false,
            check_models: false,
        }
    }
}

#[derive(Default)]
#[cfg_attr(feature = "test", derive(serde::Deserialize, Debug, Eq, PartialEq))]
pub struct NotesInfoParams {
    pub query: String,
}

impl Serialize for NotesInfoParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("NotesInfoParams", 1)?;

        state.serialize_field(
            "query",
            &format!(
                "{}{}",
                if cfg!(feature = "test") { "" } else { "deck:" },
                self.query
            ),
        )?;
        state.end()
    }
}

#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serde::Deserialize, Debug, Eq, PartialEq))]
#[serde(untagged)]
pub enum Params {
    CreateDeck(CreateDeckParams),
    AddNote(AddNoteParams),
    NotesInfo(NotesInfoParams),
}

#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serde::Deserialize, Debug, Eq, PartialEq))]
pub struct ApiRequest {
    pub action: ApiMethod,
    pub version: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Params>,
}

impl ApiRequest {
    pub fn make_deck_names_req() -> Self {
        ApiRequest {
            action: ApiMethod::DeckNames,
            version: API_VERSION,
            params: None,
        }
    }

    pub fn make_deck_names_and_ids_req() -> Self {
        ApiRequest {
            action: ApiMethod::DeckNamesAndIds,
            version: API_VERSION,
            params: None,
        }
    }

    pub fn make_create_deck_req(deck: &str) -> Self {
        ApiRequest {
            action: ApiMethod::CreateDeck,
            version: API_VERSION,
            params: Some(Params::CreateDeck(CreateDeckParams {
                deck: deck.to_string(),
            })),
        }
    }

    pub fn make_add_note_req(note: Note) -> Self {
        ApiRequest {
            action: ApiMethod::AddNote,
            version: API_VERSION,
            params: Some(Params::AddNote(AddNoteParams { note })),
        }
    }

    pub fn make_notes_info_req(deck: &str) -> Self {
        ApiRequest {
            action: ApiMethod::NotesInfo,
            version: API_VERSION,
            params: Some(Params::NotesInfo(NotesInfoParams {
                query: deck.to_string(),
            })),
        }
    }
}
