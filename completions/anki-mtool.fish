# Fish shell completions for anki-mtool
# Place this file in ~/.config/fish/completions/ or /usr/share/fish/completions/

# Clear existing completions
complete -c anki-mtool -e

# Main commands
complete -c anki-mtool -f -n "__fish_use_subcommand" -a "info" -d "Show description about anki-multitool"
complete -c anki-mtool -f -n "__fish_use_subcommand" -a "version" -d "Show version of anki-multitool"
complete -c anki-mtool -f -n "__fish_use_subcommand" -a "decklist" -d "List all decks in your Anki collection"
complete -c anki-mtool -f -n "__fish_use_subcommand" -a "newdeck" -d "Create a new deck in Anki"
complete -c anki-mtool -f -n "__fish_use_subcommand" -a "json2deck" -d "Import a deck from a JSON file into Anki"
complete -c anki-mtool -f -n "__fish_use_subcommand" -a "deck2json" -d "Export a deck from Anki to a JSON file"
complete -c anki-mtool -f -n "__fish_use_subcommand" -a "md2deck" -d "Import a deck from a Markdown file into Anki"
complete -c anki-mtool -f -n "__fish_use_subcommand" -a "deck2md" -d "Export a deck from Anki to a Markdown file"

# File completions for commands that take file paths
complete -c anki-mtool -f -n "__fish_seen_subcommand_from json2deck" -a "(__fish_complete_suffix .json)" -d "JSON file"
complete -c anki-mtool -f -n "__fish_seen_subcommand_from md2deck" -a "(__fish_complete_suffix .md)" -d "Markdown file"

# Dynamic deck name completion function (requires Anki to be running)
function __anki_mtool_complete_decks
    # Try to get deck list from anki-mtool, suppress errors if Anki is not running
    anki-mtool decklist 2>/dev/null | grep -E "^\s*-\s+" | sed 's/^\s*-\s*//' | sed 's/\s*$//'
end

# Deck name completions for commands that need deck names
complete -c anki-mtool -f -n "__fish_seen_subcommand_from newdeck deck2json deck2md" -a "(__anki_mtool_complete_decks)" -d "Anki deck"