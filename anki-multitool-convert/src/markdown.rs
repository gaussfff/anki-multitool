use anyhow::{Result, anyhow};
use std::{
    fs::File,
    io::{BufReader, Lines},
    path::{Path, PathBuf},
    str::FromStr,
};

use anki_multitool_ds::card::Card;
use anki_multitool_util::commit::FileCommitBuffer;

#[derive(PartialEq, Eq)]
enum TypeList {
    Ordered,
    Unordered,
    Undefined,
}

impl TypeList {
    fn is_ordered(&self) -> bool {
        *self == TypeList::Ordered
    }

    fn is_unordered(&self) -> bool {
        *self == TypeList::Unordered
    }
}

#[derive(PartialEq, Eq)]
enum ParserState {
    InList,
    InListItem,
    Undefined,
}

impl ParserState {
    fn in_list(&self) -> bool {
        *self == ParserState::InList
    }

    fn in_list_item(&self) -> bool {
        *self == ParserState::InListItem
    }
}

struct MarkdownListStream {
    lines: Lines<BufReader<File>>,
    type_list: TypeList,
}

impl MarkdownListStream {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        use std::io::BufRead;

        if !path.as_ref().exists() {
            return Err(anyhow!("file {} doesn't exist", path.as_ref().display()));
        }

        Ok(Self {
            lines: BufReader::new(File::open(path)?).lines(),
            type_list: TypeList::Undefined,
        })
    }
}

impl Iterator for MarkdownListStream {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        use pulldown_cmark::{Event, Parser, Tag, TagEnd};

        loop {
            let mut parser_state = ParserState::Undefined;
            let mut text_buffer = String::new();

            if let Some(Ok(line)) = self.lines.next() {
                // Skip empty lines
                if line.trim().is_empty() {
                    continue;
                }

                let mut events_found = false;
                for event in Parser::new(&line) {
                    events_found = true;
                    match event {
                        Event::Start(Tag::List(Some(_))) => {
                            if self.type_list.is_unordered() {
                                return Some(Err(anyhow!(
                                    "mixed or nested list are not supported"
                                )));
                            }

                            parser_state = ParserState::InList;
                            self.type_list = TypeList::Ordered;
                        }
                        Event::Start(Tag::List(None)) => {
                            if self.type_list.is_ordered() {
                                return Some(Err(anyhow!(
                                    "mixed or nested list are not supported"
                                )));
                            }

                            parser_state = ParserState::InList;
                            self.type_list = TypeList::Unordered;
                        }
                        Event::Start(Tag::Item) if parser_state.in_list() => {
                            parser_state = ParserState::InListItem;
                        }
                        Event::Text(ref text) if parser_state.in_list_item() => {
                            text_buffer.push_str(text);
                        }
                        Event::End(TagEnd::Item) if parser_state.in_list_item() => {
                            return Some(Ok(text_buffer));
                        }
                        _ => {
                            return Some(Err(anyhow!("unsupported format of markdown")));
                        }
                    }
                }

                // If no events were found for this line, it might be whitespace or other content
                // Continue to next line instead of terminating
                if !events_found {
                    continue;
                }
            } else {
                // No more lines to read
                return None;
            }
        }
    }
}

pub struct FromMarkdownDeck {
    path: PathBuf,
}

impl FromMarkdownDeck {
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
        .exec_and_commit(MarkdownListStream::new(&self.path)?)
        .await
    }
}

pub struct ToMarkdownDeck {
    path: PathBuf,
}

impl ToMarkdownDeck {
    pub fn new(deck: &str) -> Self {
        Self {
            path: PathBuf::from(format!("{deck}.md")),
        }
    }

    pub async fn write(&self, cards: impl Iterator<Item = Card>) -> Result<String> {
        use std::io::{BufWriter, Write};
        use std::sync::{
            Arc, Mutex,
            atomic::{AtomicUsize, Ordering},
        };

        if self.path.exists() {
            return Err(anyhow::anyhow!(
                "file {} already exists",
                self.path.display()
            ));
        }

        let writer = Arc::new(Mutex::new(BufWriter::new(File::create(&self.path)?)));
        let counter = Arc::new(AtomicUsize::new(1));

        FileCommitBuffer::new(
            async |data| Ok(data),
            async |data| {
                let writer = Arc::clone(&writer);
                let counter = Arc::clone(&counter);
                let card = Card::from_str(data.as_str())?;

                writeln!(
                    match writer.lock() {
                        Ok(w) => w,
                        Err(_) => return Err(anyhow::anyhow!("failed to lock writer")),
                    },
                    "{}. {} - {}",
                    counter.load(Ordering::Relaxed),
                    card.front,
                    card.back
                )?;

                counter.fetch_add(1, Ordering::Relaxed);

                Ok(())
            },
        )?
        .exec_and_commit(cards)
        .await?;

        self.path
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("failed to convert path to string"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;
    use std::{
        fs::{metadata, read_to_string},
        io::{BufReader, Seek, Write},
        sync::{Arc, Mutex},
    };
    use tempfile::NamedTempFile;

    #[tokio::test]
    pub async fn test_from_markdown_deck_ordered_list() {
        let md_file = NamedTempFile::new().expect("failed to create temp file");
        let file = Arc::new(Mutex::new(
            NamedTempFile::new().expect("failed to create temp file"),
        ));

        writeln!(
            &md_file,
            r#"1. Q - A
2. Which color? - Blue
3. ABCD? - Yes, EFGH
4. 2+2 ? - 4
5. pi? - It's definitely 3.14..."#
        )
        .expect("failed to write to temp file");

        FromMarkdownDeck::new(md_file.path())
            .expect("failed to create FromMarkdownDeck")
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
    pub async fn test_from_markdown_deck_unordered_list() {
        use std::io::{BufReader, Seek};

        let md_file = NamedTempFile::new().expect("failed to create temp file");
        let file = Arc::new(Mutex::new(
            NamedTempFile::new().expect("failed to create temp file"),
        ));

        writeln!(
            &md_file,
            r#"- Q - A
- Which color? - Blue
- ABCD? - Yes, EFGH
- 2+2 ? - 4
- pi? - It's definitely 3.14..."#
        )
        .expect("failed to write to temp file");

        FromMarkdownDeck::new(md_file.path())
            .expect("failed to create FromMarkdownDeck")
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
    pub async fn test_failed_from_markdown_deck_ordered_list() {
        let md_file = NamedTempFile::new().expect("failed to create temp file");
        let file = Arc::new(Mutex::new(
            NamedTempFile::new().expect("failed to create temp file"),
        ));

        writeln!(
            &md_file,
            r#"1. Q - A
2. Which color? - Blue
3 ABCD? - Yes, EFGH
4. 2+2 ? - 4
5. pi? - It's definitely 3.14..."#
        )
        .expect("failed to write to temp file");

        assert!(
            FromMarkdownDeck::new(md_file.path())
                .expect("failed to create FromMarkdownDeck")
                .for_each(async |data| {
                    writeln!(
                        Arc::clone(&file).lock().expect("failed to get file"),
                        "{data}"
                    )?;
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
    pub async fn test_failed_from_markdown_deck_unordered_list() {
        let md_file = NamedTempFile::new().expect("failed to create temp file");
        let file = Arc::new(Mutex::new(
            NamedTempFile::new().expect("failed to create temp file"),
        ));

        writeln!(
            &md_file,
            r#"- Q - A
- Which color? - Blue
- ABCD? - Yes, EFGH
  2+2 ? - 4
- pi? - It's definitely 3.14..."#
        )
        .expect("failed to write to temp file");

        assert!(
            FromMarkdownDeck::new(md_file.path())
                .expect("failed to create FromMarkdownDeck")
                .for_each(async |data| {
                    writeln!(
                        Arc::clone(&file).lock().expect("failed to get file"),
                        "{data}"
                    )?;
                    Ok(())
                })
                .await
                .is_err()
        );

        assert_eq!(
            metadata(file.lock().expect("faield to get file").path())
                .expect("failed to get metadata of file")
                .len(),
            0
        );
    }

    #[tokio::test]
    pub async fn test_to_markdown_deck() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("failed to create temp directory");
        let temp_path = temp_dir.path().join("test_deck.md");

        let cards = vec![
            Card::from_str("Q - A").expect("failed to create card"),
            Card::from_str("Which color? - Blue").expect("failed to create card"),
            Card::from_str("ABCD? - Yes, EFGH").expect("failed to create card"),
            Card::from_str("2+2 ? - 4").expect("failed to create card"),
            Card::from_str("pi? - It's definitely 3.14...").expect("failed to create card"),
        ];

        let mut to_markdown_deck = ToMarkdownDeck::new("test_deck");
        to_markdown_deck.path = temp_path.clone();

        to_markdown_deck
            .write(cards.into_iter())
            .await
            .expect("failed to write card to Markdown file");

        assert!(temp_path.exists());

        assert_eq!(
            read_to_string(&temp_path).expect("failed to read file"),
            r#"1. Q - A
2. Which color? - Blue
3. ABCD? - Yes, EFGH
4. 2+2 ? - 4
5. pi? - It's definitely 3.14...
"#
        )
    }

    #[tokio::test]
    pub async fn test_failed_to_markdown_deck() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("failed to create temp directory");
        let temp_path = temp_dir.path().join("failed_test_deck.md");

        File::create(&temp_path).expect("failed to create Markdown file");

        let mut to_markdown_deck = ToMarkdownDeck::new("failed_test_deck");
        to_markdown_deck.path = temp_path;

        assert!(
            to_markdown_deck
                .write(vec![Card::default()].into_iter())
                .await
                .is_err()
        );
    }

    #[test]
    pub fn test_markdown_stream_ordered_list() {
        let md_file = NamedTempFile::new().expect("failed to create temp file");
        writeln!(
            &md_file,
            r#"1. Q - A
2. Which color? - Blue
3. ABCD? - Yes, EFGH
4. 2+2 ? - 4
5. pi? - It's definitely 3.14...
"#
        )
        .expect("faield to write to temp file");

        let stream =
            MarkdownListStream::new(md_file.path()).expect("failed to create MarkdownListStream");

        assert_eq!(
            stream
                .collect::<Result<Vec<_>>>()
                .expect("failed to collect stream"),
            vec![
                "Q - A".to_string(),
                "Which color? - Blue".to_string(),
                "ABCD? - Yes, EFGH".to_string(),
                "2+2 ? - 4".to_string(),
                "pi? - It's definitely 3.14...".to_string(),
            ]
        );
    }

    #[test]
    pub fn test_markdown_stream_unordered_list() {
        let md_file = NamedTempFile::new().expect("failed to create temp file");
        writeln!(
            &md_file,
            r#"- Q - A
- Which color? - Blue
- ABCD? - Yes, EFGH
- 2+2 ? - 4
- pi? - It's definitely 3.14...
"#
        )
        .expect("faield to write to temp file");

        let stream =
            MarkdownListStream::new(md_file.path()).expect("failed to create MarkdownListStream");

        assert_eq!(
            stream
                .collect::<Result<Vec<_>>>()
                .expect("failed to collect stream"),
            vec![
                "Q - A".to_string(),
                "Which color? - Blue".to_string(),
                "ABCD? - Yes, EFGH".to_string(),
                "2+2 ? - 4".to_string(),
                "pi? - It's definitely 3.14...".to_string(),
            ]
        );
    }

    #[test]
    pub fn test_failed_markdown_stream_ordered_list() {
        let md_file = NamedTempFile::new().expect("failed to create temp file");
        writeln!(
            &md_file,
            r#"1. Q - A
2. Which color? - Blue
- ABCD? - Yes, EFGH
3. 2+2 ? - 4
4. pi? - It's definitely 3.14...
"#
        )
        .expect("faield to write to temp file");

        let mut stream =
            MarkdownListStream::new(md_file.path()).expect("failed to create MarkdownListStream");
        assert!(stream.collect::<Result<Vec<_>>>().is_err());

        md_file
            .as_file()
            .set_len(0)
            .expect("failed to clear temp file");
        writeln!(
            &md_file,
            r#"     1. Q - A
2. Which color? - Blue
3. ABCD? - Yes, EFGH
4. 2+2 ? - 4
5. pi? - It's definitely 3.14...
"#
        )
        .expect("faield to write to temp file");

        stream =
            MarkdownListStream::new(md_file.path()).expect("failed to create MarkdownListStream");
        assert!(stream.collect::<Result<Vec<_>>>().is_err());

        md_file
            .as_file()
            .set_len(0)
            .expect("failed to clear temp file");
        writeln!(
            &md_file,
            r#"1. Q - A
2. Which color? - Blue
    - ABCD? - Yes, EFGH
    - ABCD? - Yes, EFGH
    - ABCD? - Yes, EFGH
3. 2+2 ? - 4
4. pi? - It's definitely 3.14...
"#
        )
        .expect("faield to write to temp file");

        stream =
            MarkdownListStream::new(md_file.path()).expect("failed to create MarkdownListStream");
        assert!(stream.collect::<Result<Vec<_>>>().is_err());

        md_file
            .as_file()
            .set_len(0)
            .expect("failed to clear temp file");
        writeln!(
            &md_file,
            r#"# TITLE
1. Q - A
2. Which color? - Blue
3. ABCD? - Yes, EFGH
4. 2+2 ? - 4
5. pi? - It's definitely 3.14...
"#
        )
        .expect("faield to write to temp file");

        stream =
            MarkdownListStream::new(md_file.path()).expect("failed to create MarkdownListStream");
        assert!(stream.collect::<Result<Vec<_>>>().is_err());
    }

    #[test]
    pub fn test_failed_markdown_stream_unordered_list() {
        let md_file = NamedTempFile::new().expect("failed to create temp file");
        writeln!(
            &md_file,
            r#"- Q - A
- Which color? - Blue
3. ABCD? - Yes, EFGH
- 2+2 ? - 4
- pi? - It's definitely 3.14...
"#
        )
        .expect("faield to write to temp file");

        let mut stream =
            MarkdownListStream::new(md_file.path()).expect("failed to create MarkdownListStream");
        assert!(stream.collect::<Result<Vec<_>>>().is_err());

        md_file
            .as_file()
            .set_len(0)
            .expect("failed to clear temp file");
        writeln!(
            &md_file,
            r#"     - Q - A
- Which color? - Blue
- ABCD? - Yes, EFGH
- 2+2 ? - 4
- pi? - It's definitely 3.14...
"#
        )
        .expect("faield to write to temp file");

        stream =
            MarkdownListStream::new(md_file.path()).expect("failed to create MarkdownListStream");
        assert!(stream.collect::<Result<Vec<_>>>().is_err());

        md_file
            .as_file()
            .set_len(0)
            .expect("failed to clear temp file");
        writeln!(
            &md_file,
            r#"- Q - A
- Which color? - Blue
    1. ABCD? - Yes, EFGH
    2. ABCD? - Yes, EFGH
    3. ABCD? - Yes, EFGH
- 2+2 ? - 4
- pi? - It's definitely 3.14...
"#
        )
        .expect("faield to write to temp file");

        stream =
            MarkdownListStream::new(md_file.path()).expect("failed to create MarkdownListStream");
        assert!(stream.collect::<Result<Vec<_>>>().is_err());

        md_file
            .as_file()
            .set_len(0)
            .expect("failed to clear temp file");
        writeln!(
            &md_file,
            r#"# TITLE
- Q - A
- Which color? - Blue
- ABCD? - Yes, EFGH
- 2+2 ? - 4
- pi? - It's definitely 3.14...
"#
        )
        .expect("faield to write to temp file");

        stream =
            MarkdownListStream::new(md_file.path()).expect("failed to create MarkdownListStream");
        assert!(stream.collect::<Result<Vec<_>>>().is_err());
    }
}
