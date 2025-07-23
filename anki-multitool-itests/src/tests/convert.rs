use serde_json::Value;
use std::{
    sync::LazyLock,
    fs::read_to_string
};

use anki_multitool_core::ToolController;
use anki_multitool_util::file;
use anki_multitool_test_util::{env::TestEnv, server::MockAnkiServer, with_mserver};

use crate::util;

const HOST: &str = "localhost";

static TEST_ENV: LazyLock<TestEnv> = LazyLock::new(|| TestEnv::init().unwrap());

#[tokio::test]
pub async fn test_get_deck_names() {
    let _ = &*TEST_ENV;
    let port = 8765;

    with_mserver! {
        use_port port;

        let decks = vec![
            "Test Deck 1".to_string(),
            "Test Deck 2".to_string(),
            "Test Deck 3".to_string(),
            "Test Deck 4".to_string(),
            "Test Deck 5".to_string(),
        ];

        util::load_decks(HOST, port, decks.clone()).await.expect("failed to load decks");

        let controller = ToolController::new(HOST.to_string(), port);

        let deck_names = controller
            .get_deck_names()
            .await
            .expect("failed to get deck names");

        assert_eq!(deck_names, decks);
    }
}

#[tokio::test]
pub async fn test_new_deck() {
    let _ = &*TEST_ENV;
    let port = 8766;
    let deck = "New Test Deck";

    with_mserver! {
        use_port port;

        let controller = ToolController::new(HOST.to_string(), port);

        let deck_id = controller
            .new_deck(deck)
            .await
            .expect("failed to create new deck");

        assert_eq!(deck_id, 0);

        let deck_names = controller
            .get_deck_names()
            .await
            .expect("failed to get deck names");

        assert!(deck_names.contains(&deck.to_string()));
    }
}

#[tokio::test]
pub async fn test_convert_json() {
    let _ = &*TEST_ENV;
    let port = 8767;

    let file = util::temp_json_file().expect("failed to create temp file");
    util::write_to_file(
        file.path(),
        r#"[
            {"front": "Q1", "back": "A1"},
            {"front": "Q2", "back": "A2"},
            {"front": "Q3", "back": "A3"}
        ]"#,
    )
    .expect("failed to write to file");

    with_mserver! {
        use_port port;

        let controller = ToolController::new(HOST.to_string(), port);
        let deck = file::to_file_name(file.path()).expect("failed to get file name");
        let created_deck = controller.convert_json_to_deck(file.path()).await.expect("failed to convert JSON to deck");

        assert_eq!(deck, created_deck);

        file.close().expect("failed to close file");
        let file_path = controller.convert_deck_to_json(&deck).await.expect("failed to convert deck to JSON");

        assert_eq!(
            serde_json::from_str::<Value>(
                &read_to_string(&file_path).expect("failed to read JSON file")
            ).expect("failed to parse JSON"),
            serde_json::json!([
                {"front": "Q1", "back": "A1"},
                {"front": "Q2", "back": "A2"},
                {"front": "Q3", "back": "A3"}
            ])
        );

    }
}

#[tokio::test]
pub async fn test_convert_markdown_ordered_list() {
    let _ = &*TEST_ENV;
    let port = 8768;

    let file = util::temp_md_file().expect("failed to create temp file");
    util::write_to_file(
        file.path(),
        r#"
1. Q1 - A1
2. Q2 - A2
3. Q3 - Q3
        "#,
    )
    .expect("failed to write to file");

    with_mserver! {
        use_port port;

        let controller = ToolController::new(HOST.to_string(), port);
        let deck = file::to_file_name(file.path()).expect("failed to get file name");
        let created_deck = controller.convert_md_to_deck(file.path()).await.expect("failed to convert Markdown to deck");

        assert_eq!(deck, created_deck);

        file.close().expect("failed to close file");
        let file_path = controller.convert_deck_to_md(&deck).await.expect("failed to convert deck to Markdown");

        assert_eq!(
            &read_to_string(&file_path).expect("failed to read  file"),
            r#"1. Q1 - A1
2. Q2 - A2
3. Q3 - Q3
"#
        );
    }
}

#[tokio::test]
pub async fn test_convert_markdown_unordered_list() {
    let _ = &*TEST_ENV;
    let port = 8769;

    let file = util::temp_md_file().expect("failed to create temp file");
    util::write_to_file(
        file.path(),
        r#"
- Q1 - A1
- Q2 - A2
- Q3 - Q3
        "#,
    )
    .expect("failed to write to file");

    with_mserver! {
        use_port port;

        let controller = ToolController::new(HOST.to_string(), port);
        let deck = file::to_file_name(file.path()).expect("failed to get file name");
        let created_deck = controller.convert_md_to_deck(file.path()).await.expect("failed to convert Markdown to deck");

        assert_eq!(deck, created_deck);

        file.close().expect("failed to close file");
        let file_path = controller.convert_deck_to_md(&deck).await.expect("failed to convert deck to Markdown");

        assert_eq!(
            &read_to_string(&file_path).expect("failed to read  file"),
            r#"1. Q1 - A1
2. Q2 - A2
3. Q3 - Q3
"#
        );
    }
}
