use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
#[cfg_attr(feature = "test", derive(serde::Serialize, Debug, Eq, PartialEq))]
#[serde(untagged)]
pub enum ApiResponseData {
    Names(Vec<String>),
    NamesAndIds(HashMap<String, u64>),
    Id(u64),
    NotesInfo(Vec<NotesInfoResponseData>),
}

impl ApiResponseData {
    pub fn into_names_res(self) -> Option<Vec<String>> {
        match self {
            ApiResponseData::Names(names) => Some(names),
            _ => None,
        }
    }

    pub fn into_names_and_ids_res(self) -> Option<HashMap<String, u64>> {
        match self {
            ApiResponseData::NamesAndIds(map) => Some(map),
            _ => None,
        }
    }

    pub fn into_id_res(self) -> Option<u64> {
        match self {
            ApiResponseData::Id(id) => Some(id),
            _ => None,
        }
    }

    pub fn into_notes_info_res(self) -> Option<Vec<NotesInfoResponseData>> {
        match self {
            ApiResponseData::NotesInfo(notes) => Some(notes),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
#[cfg_attr(
    feature = "test",
    derive(
        serde::Serialize,
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        PartialOrd,
        Ord
    )
)]
pub struct NotesInfoResponseData {
    #[serde(rename = "noteId")]
    pub note_id: u64,
    pub profile: String,
    pub tags: Vec<String>,
    pub fields: OutFields,
    #[serde(rename = "modelName")]
    pub model_name: String,
    #[serde(rename = "mod")]
    pub mod_data: u64,
    pub cards: Vec<u64>,
}

#[cfg(feature = "test")]
impl NotesInfoResponseData {
    pub fn new_simple(front: &str, back: &str) -> Self {
        Self {
            fields: OutFields {
                front: Data {
                    value: front.to_string(),
                    ..Data::default()
                },
                back: Data {
                    value: back.to_string(),
                    ..Data::default()
                },
            },
            ..Self::default()
        }
    }

    pub fn with_id(mut self, id: u64) -> Self {
        self.note_id = id;
        self
    }
}

#[cfg(feature = "test")]
impl From<super::request::Note> for NotesInfoResponseData {
    fn from(note: super::request::Note) -> Self {
        Self::new_simple(note.fields.front.as_str(), note.fields.back.as_str())
    }
}

#[derive(Deserialize)]
#[cfg_attr(
    feature = "test",
    derive(
        serde::Serialize,
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        Ord,
        PartialOrd
    )
)]
pub struct OutFields {
    #[serde(rename = "Front")]
    pub front: Data,
    #[serde(rename = "Back")]
    pub back: Data,
}

#[derive(Deserialize, Default)]
#[cfg_attr(
    feature = "test",
    derive(serde::Serialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)
)]
pub struct Data {
    pub value: String,
    pub order: u16,
}

#[derive(Deserialize)]
#[cfg_attr(
    feature = "test",
    derive(serde::Serialize, Debug, Default, Eq, PartialEq)
)]
pub struct ApiResponse {
    pub error: Option<String>,
    pub result: Option<ApiResponseData>,
}

impl ApiResponse {
    pub fn into_result(self) -> Result<ApiResponseData> {
        match (self.error, self.result) {
            (Some(error), _) => Err(anyhow!(error)),
            (None, Some(result)) => Ok(result),
            _ => Err(anyhow!("empty response")),
        }
    }
}

#[cfg(feature = "test")]
impl ApiResponse {
    pub fn as_error(error: String) -> Self {
        Self {
            error: Some(error),
            result: None,
        }
    }

    pub fn as_success(result: ApiResponseData) -> Self {
        Self {
            error: None,
            result: Some(result),
        }
    }

    pub fn with_names_ok_res(res: Vec<String>) -> Self {
        Self::as_success(ApiResponseData::Names(res))
    }

    pub fn with_names_and_ids_ok_res(res: HashMap<String, u64>) -> Self {
        Self::as_success(ApiResponseData::NamesAndIds(res))
    }

    pub fn with_id_ok_res(res: u64) -> Self {
        Self::as_success(ApiResponseData::Id(res))
    }

    pub fn with_notes_info_ok_res(res: Vec<NotesInfoResponseData>) -> Self {
        Self::as_success(ApiResponseData::NotesInfo(res))
    }
}
