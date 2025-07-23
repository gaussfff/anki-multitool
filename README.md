# anki-multitool (anki-mtool)
Multitool for Anki - just useful multitool for Anki users. What it can do:
- Export decks to JSON and Markdown files
- Import decks from JSON and Markdown files
- List all decks in your Anki collection
- Create a new deck 
- To be continued...

## Requirements
- [Rust 1.88.0+ (*only if you are installing from source*)](https://www.rust-lang.org/tools/install)
- [Anki 25.02.7+](https://apps.ankiweb.net/)
- [AnkiConnect plugin with API version 6+](https://ankiweb.net/shared/info/2055492159)
- [OpenSSL 3.5.0+ (*only for Linux users*)](https://openssl.org/)

## Instalation
You can install manually by cloning the repository and running the following command:
```bash
sudo chmod +x install.sh
./install.sh
```
Or mabye I'll make ready-to-use binaries in the future, so you can just download and build + run it.

## Usage
`anki-mtool <command> [options]`

### Available commands
- `anki-mtool help [command]` - show help for a command or list all commands
- `anki-mtool version` - show the version of anki-multitool
- `anki-mtool decklist` - list all decks in your Anki collection
- `anki-mtool newdeck <deck-name>` - create a new deck in Anki
- `anki-mtool json2deck <path-to-json-file>` - import a deck from a JSON file into Anki
- `anki-mtool deck2json <deck-name>` - export a deck from Anki to a JSON file
- `anki-mtool md2deck <path-to-md-file>` - import a deck from a Markdown file into Anki
- `anki-mtool deck2md <deck-name>` - export a deck from Anki to a Markdown file

## Custom host and port
You can define custom host and port for the server by setting the environment variables `ANKI_MULTITOOL_HOST` and `ANKI_MULTITOOL_PORT`. For example, you can run the following command in your terminal:
```bash
export ANKI_MULTITOOL_HOST="<cusotm-host>"
export ANKI_MULTITOOL_PORT="<custom-port>"
```

By default, tool will listening on `localhost:8765`.

## Formats
Anki-multitool supports two formats for importing and exporting decks: JSON and Markdown. Full examples you can find in `examples` directory.

### JSON format
Name of file is deck name, so if you want to export deck named "My Deck" to JSON file, it will be saved as `my_deck.json`. 
Vice versa, if you want to import deck from JSON file, the name of the deck will be taken from the file name (without extension).
If deck doesn't exist in Anki, it will be created automatically.

Here is example of JSON format used for importing and exporting decks:
```json
[
    {
      "front": "What is the capital of Urakine?",
      "back": "Kyiv"
    },
    {
      "front": "What is the capital of Germany?",
      "back": "Berlin"
    }
]
```

### Markdown format
Name of file is deck name, so if you want to export deck named "My Deck" to Markdown file, it will be saved as `my_deck.md`.
Vice versa, if you want to import deck from Markdown file, the name of the deck will be taken from the file name (without extension).
If deck doesn't exist in Anki, it will be created automatically.

Here is example of Markdown formats used for importing and exporting decks:
```markdown
1. What is the capital of Ukraine? - Kyiv
2. What is the capital of Germany? - Berlin
```

and 

```markdown
- What is the capital of Ukraine? - Kyiv
- What is the capital of Germany? - Berlin
```

## License
This software is under the MIT license. See details in license file.

## Contribution
So-so, just create issue or PR, I will try to fix it as soon as possible.
