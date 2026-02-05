#!/bin/sh
# FlagLite CLI installer
# Usage: curl -fsSL https://flaglite.dev/install.sh | sh
#
# This script detects your OS and architecture, downloads the appropriate
# FlagLite CLI binary, and installs it to ~/.local/bin (or /usr/local/bin with sudo).

set -e

REPO="faiscadev/flaglite"
BINARY_NAME="flaglite"
INSTALL_DIR="${FLAGLITE_INSTALL_DIR:-$HOME/.local/bin}"

# Colors (if terminal supports it)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

info() {
    printf "${BLUE}info${NC}: %s\n" "$1"
}

success() {
    printf "${GREEN}success${NC}: %s\n" "$1"
}

warn() {
    printf "${YELLOW}warn${NC}: %s\n" "$1"
}

error() {
    printf "${RED}error${NC}: %s\n" "$1" >&2
    exit 1
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *)       error "Unsupported operating system: $(uname -s)" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "amd64" ;;
        aarch64|arm64) echo "arm64" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# Get latest release tag
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"tag_name": *"([^"]+)".*/\1/' | \
        sed 's/^cli-//'
}

# Download and install
install() {
    OS=$(detect_os)
    ARCH=$(detect_arch)
    
    info "Detected OS: $OS, Architecture: $ARCH"
    
    # Get version (from env or latest)
    if [ -n "$FLAGLITE_VERSION" ]; then
        VERSION="$FLAGLITE_VERSION"
    else
        info "Fetching latest version..."
        VERSION=$(get_latest_version)
    fi
    
    if [ -z "$VERSION" ]; then
        error "Could not determine version to install. Check your internet connection or set FLAGLITE_VERSION."
    fi
    
    info "Installing FlagLite CLI $VERSION"
    
    # Construct download URL
    BINARY_FILENAME="${BINARY_NAME}-${OS}-${ARCH}"
    if [ "$OS" = "windows" ]; then
        BINARY_FILENAME="${BINARY_FILENAME}.exe"
    fi
    
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/cli-${VERSION}/${BINARY_FILENAME}"
    CHECKSUM_URL="https://github.com/${REPO}/releases/download/cli-${VERSION}/SHA256SUMS"
    
    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT
    
    # Download binary
    info "Downloading from $DOWNLOAD_URL..."
    if ! curl -fsSL -o "$TMP_DIR/$BINARY_NAME" "$DOWNLOAD_URL"; then
        error "Failed to download binary. Release may not exist for your platform."
    fi
    
    # Download and verify checksum (optional, warn if fails)
    info "Verifying checksum..."
    if curl -fsSL -o "$TMP_DIR/SHA256SUMS" "$CHECKSUM_URL" 2>/dev/null; then
        cd "$TMP_DIR"
        if command -v sha256sum > /dev/null 2>&1; then
            if echo "$(grep "$BINARY_FILENAME" SHA256SUMS)" | sha256sum -c - > /dev/null 2>&1; then
                success "Checksum verified"
            else
                warn "Checksum verification failed"
            fi
        elif command -v shasum > /dev/null 2>&1; then
            EXPECTED=$(grep "$BINARY_FILENAME" SHA256SUMS | awk '{print $1}')
            ACTUAL=$(shasum -a 256 "$BINARY_NAME" | awk '{print $1}')
            if [ "$EXPECTED" = "$ACTUAL" ]; then
                success "Checksum verified"
            else
                warn "Checksum verification failed"
            fi
        else
            warn "No checksum tool available, skipping verification"
        fi
        cd - > /dev/null
    else
        warn "Could not download checksums, skipping verification"
    fi
    
    # Make executable
    chmod +x "$TMP_DIR/$BINARY_NAME"
    
    # Create install directory if needed
    mkdir -p "$INSTALL_DIR"
    
    # Install
    info "Installing to $INSTALL_DIR/$BINARY_NAME..."
    if [ -w "$INSTALL_DIR" ]; then
        mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    else
        warn "Cannot write to $INSTALL_DIR, trying with sudo..."
        sudo mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    success "FlagLite CLI installed successfully!"
    
    # Check if in PATH
    if ! command -v flaglite > /dev/null 2>&1; then
        echo ""
        warn "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add it to your shell config:"
        echo ""
        echo "  # For bash (~/.bashrc)"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        echo "  # For zsh (~/.zshrc)"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        echo "  # For fish (~/.config/fish/config.fish)"
        echo "  set -gx PATH \$HOME/.local/bin \$PATH"
        echo ""
        echo "Then restart your shell or run: source ~/.bashrc (or ~/.zshrc)"
    else
        echo ""
        echo "Get started:"
        echo "  flaglite signup    # Create an account"
        echo "  flaglite --help    # See all commands"
    fi
}

# Run installer
install
