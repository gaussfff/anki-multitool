#!/bin/bash

set -e

BINARY_NAME="anki-mtool"
INSTALL_DIR="/usr/local/bin"

echo "🔨 Building anki-multitool in release mode..."
cargo build --release --bin $BINARY_NAME

echo "📦 Binary built successfully at target/release/$BINARY_NAME"

if [ ! -f "target/release/$BINARY_NAME" ]; then
    echo "❌ Build failed: Binary not found at target/release/$BINARY_NAME"
    exit 1
fi

echo "🔧 Making binary executable..."
chmod +x "target/release/$BINARY_NAME"

echo "🔐 Installing $BINARY_NAME to $INSTALL_DIR..."

if [ "$EUID" -eq 0 ]; then
    echo "🚀 Running as root, installing directly..."
    cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
    echo "✅ Successfully installed $BINARY_NAME to $INSTALL_DIR"
elif command -v sudo >/dev/null 2>&1; then
    echo "📝 This requires administrator privileges."
    echo "You will be prompted for your password to install to $INSTALL_DIR"
    echo ""
    if sudo cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"; then
        echo "✅ Successfully installed $BINARY_NAME to $INSTALL_DIR"
    else
        echo "❌ Installation failed. You can manually install with:"
        echo "   sudo cp target/release/$BINARY_NAME $INSTALL_DIR/"
        exit 1
    fi
else
    echo "⚠️  sudo not available. Please install manually with root privileges:"
    echo "   cp target/release/$BINARY_NAME $INSTALL_DIR/"
    echo ""
    echo "Or add the binary location to your PATH:"
    echo "   export PATH=\"\$PATH:\$(pwd)/target/release\""
    echo "   # Add the above line to your ~/.bashrc or ~/.zshrc"
    exit 1
fi

# Install fish completions if fish is available
install_fish_completions() {
    local user_fish_dir="$HOME/.config/fish/completions"
    local system_fish_dir="/usr/share/fish/completions"
    local completion_file="completions/anki-mtool.fish"
    
    if [ ! -f "$completion_file" ]; then
        echo "⚠️  Fish completion file not found at $completion_file"
        return 1
    fi
    
    # Try user directory first
    if [ -d "$user_fish_dir" ]; then
        if cp "$completion_file" "$user_fish_dir/"; then
            echo "✅ Fish completions installed to $user_fish_dir"
            return 0
        fi
    fi
    
    # Try system directory with sudo
    if [ -d "$system_fish_dir" ] && command -v sudo >/dev/null 2>&1; then
        echo "📝 Installing fish completions system-wide requires administrator privileges."
        if sudo cp "$completion_file" "$system_fish_dir/"; then
            echo "✅ Fish completions installed to $system_fish_dir"
            return 0
        fi
    fi
    
    echo "❌ Failed to install fish completions automatically."
    echo "You can manually install with:"
    echo "   cp $completion_file ~/.config/fish/completions/"
    return 1
}

# Check if fish is installed and offer to install completions
if command -v fish >/dev/null 2>&1; then
    echo ""
    echo "🐟 Fish shell detected!"
    echo "Would you like to install fish shell completions for better autocompletion? (y/n)"
    read -r response
    case "$response" in
        [yY][eE][sS]|[yY])
            echo "🔧 Installing fish completions..."
            install_fish_completions
            ;;
        *)
            echo "⏭️  Skipping fish completions installation."
            echo "You can manually install later by copying:"
            echo "   cp completions/anki-mtool.fish ~/.config/fish/completions/"
            ;;
    esac
else
    echo ""
    echo "ℹ️  Fish shell completions are available in completions/anki-mtool.fish"
    echo "Install fish shell and copy the file to ~/.config/fish/completions/ for autocompletion."
fi

echo ""
echo "🎉 Installation complete!"
echo "You can now run '$BINARY_NAME' from anywhere in your terminal."
echo ""
echo "Try running: $BINARY_NAME --help"
echo ""
echo "To uninstall later, run: sudo rm $INSTALL_DIR/$BINARY_NAME"
