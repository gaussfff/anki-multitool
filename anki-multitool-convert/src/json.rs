use anyhow::{Result, anyhow};
use serde::de::DeserializeOwned;
use std::{
    fs::File,
    marker::PhantomData,
    path::{Path, PathBuf},
    str::FromStr,
};

use anki_multitool_ds::card::Card;
use anki_multitool_util::commit::FileCommitBuffer;

struct JsonArrayStream<T>
where
    T: DeserializeOwned,
{
    file: File,
    in_array: bool,
    in_object: bool,
    _type: PhantomData<T>,
}

impl<T: DeserializeOwned> JsonArrayStream<T> {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            file: File::open(path)?,
            in_array: false,
            in_object: false,
            _type: PhantomData,
        })
    }
}

impl<T: DeserializeOwned> Iterator for JsonArrayStream<T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        use std::io::{ErrorKind, Read};

        let mut buffer = [0u8];
        let mut str_buffer = String::new();

        loop {
            match self.file.read_exact(&mut buffer) {
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                    if self.in_array {
                        return Some(Err(anyhow!("unexpected end of JSON array")));
                    } else if self.in_object {
                        return Some(Err(anyhow!("unexpected end of JSON object")));
                    } else {
                        return None;
                    }
                }
                Err(e) => {
                    return Some(Err(e.into()));
                }
                Ok(_) => match buffer[0] {
                    ws if ws.is_ascii_whitespace() => {
                        if self.in_array {
                            str_buffer.push(ws as char);
                        }
                    }
                    b'[' => {
                        self.in_array = true;
                    }
                    b']' if self.in_array => {
                        self.in_array = false;
                    }
                    b @ b',' if self.in_array => {
                        if self.in_object {
                            str_buffer.push(b as char);
                        }
                    }
                    b @ b'{' if self.in_array => {
                        self.in_object = true;
                        str_buffer.push(b as char);
                    }
                    b @ b'}' if self.in_object => {
                        self.in_object = false;
                        str_buffer.push(b as char);
                        return Some(serde_json::from_str(&str_buffer).map_err(|e| e.into()));
                    }
                    b => {
                        if self.in_array {
                            str_buffer.push(b as char);
                        } else {
                            return Some(Err(anyhow!(
                                "unexpected {} character in JSON array",
                                b as char
                            )));
                        }
                    }
                },
            }
        }
    }
}

pub struct FromJsonDeck {
    path: PathBuf,
}

impl FromJsonDeck {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
        })
    }

    pub async fn for_each<A>(&self, action: A) -> Result<()>
    where
        A: AsyncFn(Card) -> Result<()>,
    {
        FileCommitBuffer::new(
            async |data| data,
            async |data| action(Card::from_str(data.as_str())?).await,
        )?
        .exec_and_commit(JsonArrayStream::<Card>::new(&self.path)?)
        .await
    }
}

pub struct ToJsonDeck {
    path: PathBuf,
}

impl ToJsonDeck {
    pub fn new(deck: &str) -> Self {
        Self {
            path: PathBuf::from(format!("{deck}.json")),
        }
    }

    pub async fn write(&self, cards: impl Iterator<Item = Card>) -> Result<String> {
        use serde::{Serializer, ser::SerializeSeq};
        use serde_json::Serializer as JsonSerializer;
        use std::{
            io::BufWriter,
            sync::{Arc, Mutex},
        };

        if self.path.exists() {
            return Err(anyhow!("file {} already exists", self.path.display()));
        }

        let mut serializer = JsonSerializer::new(BufWriter::new(File::create(&self.path)?));
        let seq = Arc::new(Mutex::new(serializer.serialize_seq(None)?));

        FileCommitBuffer::new(
            async |data| Ok(data),
            async |data| {
                Arc::clone(&seq)
                    .lock()
                    .map_err(|_| anyhow!("failed to lock serializer"))?
                    .serialize_element(&Card::from_str(data.as_str())?)
                    .map_err(|e| e.into())
            },
        )?
        .exec_and_commit(cards)
        .await?;

        //TODO: maybe we should find better solution
        Arc::try_unwrap(seq)
            .map_err(|_| anyhow!("failed to unwrap Arc"))?
            .into_inner()
            .map_err(|_| anyhow!("failed to unlock mutex"))?
            .end()?;

        self.path
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("failed to convert path to string"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;
    use std::{
        fs::read_to_string,
        io::Write,
        sync::{Arc, Mutex},
    };
    use tempfile::NamedTempFile;

    #[tokio::test]
    pub async fn test_from_json_deck() {
        use std::io::{BufReader, Seek};

        let json_file = NamedTempFile::new().expect("failed to create temp file");
        let file = Arc::new(Mutex::new(
            NamedTempFile::new().expect("failed to create temp file"),
        ));

        writeln!(
            &json_file,
            r#"[
                {{
                    "front": "Q",
                    "back": "A"
                }},
                {{
                    "front": "Which color?",
                    "back": "Blue"
                }},
                {{
                    "front": "ABCD?",
                    "back": "Yes, EFGH"
                }},
                {{
                    "front": "2+2 ?",
                    "back": "4"
                }},
                {{
                    "front": "pi?",
                    "back": "It's definitely 3.14..."
                }}
            ]"#
        )
        .expect("failed to write to temp file");

        FromJsonDeck::new(json_file.path())
            .expect("failed to create FromJHsonDeck instance")
            .for_each(async |data| {
                writeln!(
                    Arc::clone(&file).lock().expect("failed to get file"),
                    "{data}"
                )?;
                Ok(())
            })
            .await
            .expect("failed to process cards");

        file.lock()
            .expect("failed to get file")
            .rewind()
            .expect("failed to rewind temp file");
        let file_guard = file.lock().expect("failed to get file");
        let mut lines = BufReader::new(file_guard.as_file()).lines();

        assert_eq!(lines.next().unwrap().unwrap(), "Q - A");
        assert_eq!(lines.next().unwrap().unwrap(), "Which color? - Blue");
        assert_eq!(lines.next().unwrap().unwrap(), "ABCD? - Yes, EFGH");
        assert_eq!(lines.next().unwrap().unwrap(), "2+2 ? - 4");
        assert_eq!(
            lines.next().unwrap().unwrap(),
            "pi? - It's definitely 3.14..."
        );
        assert!(lines.next().is_none(), "expected no more lines in the file");
    }

    #[tokio::test]
    pub async fn test_failed_from_json_deck() {
        use std::fs::metadata;

        let json_file = NamedTempFile::new().expect("failed to create temp file");
        let file = Arc::new(Mutex::new(
            NamedTempFile::new().expect("failed to create temp file"),
        ));

        // JSON syntax error, check 3th object, value of back field
        writeln!(
            &json_file,
            r#"[
                {{
                    "front": "Q",
                    "back": "A"
                }},
                {{
                    "front": "Which color?",
                    "back": "Blue"
                }},
                {{
                    "front": "ABCD?",
                    "back": "Yes, EFGH 
                }},
                {{
                    "front": "2+2 ?",
                    "back": "4"
                }},
                {{
                    "front": "pi?",
                    "back": "It's definitely 3.14..."
                }}
            ]"#
        )
        .expect("failed to write to temp file");

        assert!(
            FromJsonDeck::new(json_file.path())
                .expect("failed to create FromJHsonDeck instance")
                .for_each(async |data| {
                    let file = Arc::clone(&file);
                    writeln!(file.lock().expect("failed to get file"), "{data}")
                        .expect("failed to write card to file");
                    Ok(())
                })
                .await
                .is_err()
        );

        assert_eq!(
            metadata(file.lock().expect("failed to get file").path())
                .expect("failed to get metadata of file")
                .len(),
            0
        );
    }

    #[tokio::test]
    pub async fn test_to_json_deck() {
        use serde_json::Value;
        use tempfile::tempdir;

        let cards = vec![
            Card {
                front: "Q".to_string(),
                back: "A".to_string(),
            },
            Card {
                front: "Which color?".to_string(),
                back: "Blue".to_string(),
            },
            Card {
                front: "ABCD?".to_string(),
                back: "Yes, EFGH".to_string(),
            },
            Card {
                front: "2+2 ?".to_string(),
                back: "4".to_string(),
            },
            Card {
                front: "pi?".to_string(),
                back: "It's definitely 3.14...".to_string(),
            },
        ];

        let temp_dir = tempdir().expect("failed to create temp directory");
        let temp_path = temp_dir.path().join("test_deck.json");

        let mut to_json_deck = ToJsonDeck::new("test_deck");
        to_json_deck.path = temp_path.clone();

        to_json_deck
            .write(cards.into_iter())
            .await
            .expect("failed to write cards to JSON file");

        assert!(temp_path.exists());

        assert_eq!(
            serde_json::from_str::<Value>(
                read_to_string(&temp_path)
                    .expect("failed to read JSON file")
                    .as_str()
            )
            .expect("failed to parse JSON"),
            serde_json::from_str::<Value>(
                r#"[
                {"front": "Q", "back": "A"},
                {"front": "Which color?", "back": "Blue"},
                {"front": "ABCD?", "back": "Yes, EFGH"},
                {"front": "2+2 ?", "back": "4"},
                {"front": "pi?", "back": "It's definitely 3.14..."}
            ]"#
            )
            .expect("failed to parse expected JSON")
        );
    }

    #[tokio::test]
    pub async fn test_failed_to_json_deck() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("failed to create temp directory");
        let temp_path = temp_dir.path().join("failed_test_deck.json");

        File::create(&temp_path).expect("failed to create JSON file");

        let mut to_json_deck = ToJsonDeck::new("failed_test_deck");
        to_json_deck.path = temp_path;

        assert!(to_json_deck.write(vec![].into_iter()).await.is_err());
    }

    #[test]
    pub fn test_json_array_stream() {
        let json_file = NamedTempFile::new().expect("failed to create temp file");
        writeln!(
            &json_file,
            r#"[
                {{
                    "front": "Q",
                    "back": "A"
                }},
                {{
                    "front": "Which color?",
                    "back": "Blue"
                }},
                {{
                    "front": "ABCD?",
                    "back": "Yes, EFGH"
                }},
                {{
                    "front": "2+2 ?",
                    "back": "4"
                }},
                {{
                    "front": "pi?",
                    "back": "It's definitely 3.14..."
                }}
            ]"#
        )
        .expect("failed to write to temp file");

        let stream = JsonArrayStream::<Card>::new(json_file.path())
            .expect("failed to create JsonArrayStream");

        assert_eq!(
            stream
                .collect::<Result<Vec<_>>>()
                .expect("failed to collect stream"),
            vec![
                Card {
                    front: "Q".to_string(),
                    back: "A".to_string(),
                },
                Card {
                    front: "Which color?".to_string(),
                    back: "Blue".to_string(),
                },
                Card {
                    front: "ABCD?".to_string(),
                    back: "Yes, EFGH".to_string(),
                },
                Card {
                    front: "2+2 ?".to_string(),
                    back: "4".to_string(),
                },
                Card {
                    front: "pi?".to_string(),
                    back: "It's definitely 3.14...".to_string(),
                }
            ]
        );
    }

    #[test]
    pub fn test_failed_json_array_stream() {
        let json_file = NamedTempFile::new().expect("failed to create temp file");
        writeln!(
            &json_file,
            r#"[
                {{
                    "front": "Q,
                    "back": "A"
                }}
            ]"#
        )
        .expect("failed to write to temp file");

        let mut stream = JsonArrayStream::<Card>::new(json_file.path())
            .expect("failed to create JsonArrayStream");

        assert!(stream.collect::<Result<Vec<_>>>().is_err());

        json_file
            .as_file()
            .set_len(0)
            .expect("failed to clear temp file");
        writeln!(
            &json_file,
            r#"[
                {{
                    "front": "Q",
                    "back": "A"
                }}
                {{
                    "front": "Q",
                    "back": "A"
                }}
            ]"#
        )
        .expect("failed to write to temp file");
        stream = JsonArrayStream::<Card>::new(json_file.path())
            .expect("failed to create JsonArrayStream");

        assert!(stream.collect::<Result<Vec<_>>>().is_err());

        json_file
            .as_file()
            .set_len(0)
            .expect("failed to clear temp file");
        writeln!(
            &json_file,
            r#"[
                {{
                    "front": "Q",
                    "back": ["A"]
                }},
                {{
                    "front": "Q",
                    "back": "A"
                }}
            ]"#
        )
        .expect("failed to write to temp file");
        stream = JsonArrayStream::<Card>::new(json_file.path())
            .expect("failed to create JsonArrayStream");

        assert!(stream.collect::<Result<Vec<_>>>().is_err());

        json_file
            .as_file()
            .set_len(0)
            .expect("failed to clear temp file");
        writeln!(
            &json_file,
            r#"[
                {{
                    "front": "Q",
                    "back": {{"A": "A"}}
                }},
                {{
                    "front": "Q",
                    "back": "A"
                }}
            ]"#
        )
        .expect("failed to write to temp file");
        stream = JsonArrayStream::<Card>::new(json_file.path())
            .expect("failed to create JsonArrayStream");

        assert!(stream.collect::<Result<Vec<_>>>().is_err());

        json_file
            .as_file()
            .set_len(0)
            .expect("failed to clear temp file");
        writeln!(
            &json_file,
            r#"[
                {{
                    "front": "Q",
                    "back": "A"
                }},
                {{
                    "front": "Q",
                    "back": "A"
                
            ]"#
        )
        .expect("failed to write to temp file");
        stream = JsonArrayStream::<Card>::new(json_file.path())
            .expect("failed to create JsonArrayStream");

        assert!(stream.collect::<Result<Vec<_>>>().is_err());

        json_file
            .as_file()
            .set_len(0)
            .expect("failed to clear temp file");
        writeln!(
            &json_file,
            r#"[
                {{
                    "front": "Q",
                    "back": "A"
                }},
                {{
                    "front": "Q",
                    "back": "A"
                }}
            "#
        )
        .expect("failed to write to temp file");
        stream = JsonArrayStream::<Card>::new(json_file.path())
            .expect("failed to create JsonArrayStream");

        assert!(stream.collect::<Result<Vec<_>>>().is_err());
    }
}
