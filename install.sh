#!/usr/bin/env bash
# ==============================================================================
# Bit PDF Reader — Universal Linux Installer
# Repository: https://github.com/MI-Syafaudin/bit-pdf-reader
# Author: MI_Syafaudin
# License: Apache-2.0
# ==============================================================================

set -euo pipefail

REPO_OWNER="MI-Syafaudin"
REPO_NAME="bit-pdf-reader"
REPO_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}"
VERSION="0.1.0"

# ANSI Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m' # No Color

print_banner() {
    cat << 'EOF'
  ___  _ ___   ___  ___  ____ 
  |__] |  |    |__] |  \ |___ 
  |__] |  |    |    |__/ |    
EOF
    echo -e "${CYAN}${BOLD}Bit PDF Reader — Universal Linux Installer${NC}"
    echo -e "${BLUE}${REPO_URL}${NC}\n"
}

info() {
    echo -e "${CYAN}[INFO]${NC} $*"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

print_banner

# 1. Verify OS is Linux
OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
    error "Bit PDF Reader is designed exclusively for Linux systems. Detected OS: $OS"
    exit 1
fi

# 2. Detect CPU architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)
        TARGET_ARCH="x86_64"
        ;;
    aarch64|arm64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        error "Unsupported architecture: $ARCH. Supported architectures are x86_64 and aarch64."
        exit 1
        ;;
esac
info "Detected Architecture: $TARGET_ARCH ($OS)"

# 3. Determine Installation Destination
# Priority: /usr/local/bin if writable or sudo, otherwise ~/.local/bin
INSTALL_DIR="/usr/local/bin"
BASH_COMP_DIR="/usr/share/bash-completion/completions"
FISH_COMP_DIR="/usr/share/fish/vendor_completions.d"
ZSH_COMP_DIR="/usr/share/zsh/site-functions"
MAN_DIR="/usr/local/share/man/man1"

if [ ! -w "$INSTALL_DIR" ]; then
    if [ "$EUID" -ne 0 ]; then
        INSTALL_DIR="$HOME/.local/bin"
        BASH_COMP_DIR="$HOME/.local/share/bash-completion/completions"
        FISH_COMP_DIR="$HOME/.config/fish/completions"
        ZSH_COMP_DIR="$HOME/.local/share/zsh/site-functions"
        MAN_DIR="$HOME/.local/share/man/man1"
        info "Notice: /usr/local/bin is not writable without root. Installing to user directory: $INSTALL_DIR"
    fi
fi

# 4. Ensure target directory exists
mkdir -p "$INSTALL_DIR"
info "Target binary directory: $INSTALL_DIR"

# 5. Create temporary working directory
TMP_DIR="$(mktemp -d /tmp/bit-pdf-install.XXXXXX)"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

INSTALLED=0

# Download Helper (curl or wget)
fetch_file() {
    local url="$1"
    local output="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$output" "$url"
    else
        error "Neither 'curl' nor 'wget' was found on your system. Please install one of them."
        exit 1
    fi
}

url_exists() {
    local url="$1"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -I "$url" >/dev/null 2>&1
    elif command -v wget >/dev/null 2>&1; then
        wget --spider -q "$url" >/dev/null 2>&1
    else
        return 1
    fi
}

# 6. Installation Strategy:
# Method A: Pre-compiled GitHub Release binary tarball
RELEASE_TARBALL="bit-pdf-linux-${TARGET_ARCH}.tar.gz"
RELEASE_URL="${REPO_URL}/releases/latest/download/${RELEASE_TARBALL}"

info "Checking for pre-compiled GitHub Release binary (${RELEASE_TARBALL})..."
if url_exists "$RELEASE_URL"; then
    info "Downloading pre-compiled release from GitHub..."
    if fetch_file "$RELEASE_URL" "$TMP_DIR/$RELEASE_TARBALL"; then
        tar -xzf "$TMP_DIR/$RELEASE_TARBALL" -C "$TMP_DIR"
        if [ -f "$TMP_DIR/bit-pdf" ]; then
            chmod +x "$TMP_DIR/bit-pdf"
            install -m 755 "$TMP_DIR/bit-pdf" "$INSTALL_DIR/bit-pdf"
            if [ -f "$TMP_DIR/pdfx" ]; then
                install -m 755 "$TMP_DIR/pdfx" "$INSTALL_DIR/pdfx"
            else
                ln -sf "$INSTALL_DIR/bit-pdf" "$INSTALL_DIR/pdfx"
            fi
            INSTALLED=1
            success "Installed bit-pdf and pdfx binaries to $INSTALL_DIR!"
        fi
    fi
fi

# Method B: Direct install via Cargo from GitHub repository
if [ "$INSTALLED" -eq 0 ]; then
    if command -v cargo >/dev/null 2>&1; then
        info "Compiling and installing directly from GitHub repository via Cargo..."
        if cargo install --git "${REPO_URL}.git" pdfx-cli; then
            INSTALLED=1
            success "Successfully compiled and installed bit-pdf & pdfx via Cargo!"
        fi
    fi
fi

# Method C: If running from cloned local repository, build/install directly
if [ "$INSTALLED" -eq 0 ]; then
    if [ -f "./Cargo.toml" ] && [ -d "./crates/pdfx-cli" ] && command -v cargo >/dev/null 2>&1; then
        info "Local repository detected. Installing from current workspace..."
        if cargo install --path crates/pdfx-cli; then
            INSTALLED=1
            success "Successfully installed bit-pdf & pdfx from local workspace!"
        fi
    fi
fi

# Method D: Clone and build if git + cargo available
if [ "$INSTALLED" -eq 0 ]; then
    if command -v git >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1; then
        info "Cloning GitHub repository to build release binaries..."
        git clone --depth 1 "${REPO_URL}.git" "$TMP_DIR/repo"
        cd "$TMP_DIR/repo"
        cargo build --release --locked
        install -m 755 target/release/bit-pdf "$INSTALL_DIR/bit-pdf"
        install -m 755 target/release/pdfx "$INSTALL_DIR/pdfx"
        INSTALLED=1
        success "Successfully built and installed binaries from source!"
    fi
fi

# If all installation methods failed
if [ "$INSTALLED" -eq 0 ]; then
    echo ""
    error "Could not automatically install Bit PDF Reader."
    echo -e "${YELLOW}Please choose one of the manual GitHub installation methods:${NC}"
    echo "  1. If Rust is installed: cargo install --git ${REPO_URL}.git pdfx-cli"
    echo "  2. Clone & build:        git clone ${REPO_URL}.git && cd bit-pdf-reader && cargo build --release"
    echo "  3. Download releases:    ${REPO_URL}/releases"
    exit 1
fi

# 7. Setup Man Pages and Shell Completions if binary is in PATH or accessible
BIN_PATH=""
if command -v bit-pdf >/dev/null 2>&1; then
    BIN_PATH="bit-pdf"
elif [ -x "$INSTALL_DIR/bit-pdf" ]; then
    BIN_PATH="$INSTALL_DIR/bit-pdf"
fi

if [ -n "$BIN_PATH" ]; then
    # Generate Shell Completions
    if [ -d "$BASH_COMP_DIR" ] && [ -w "$BASH_COMP_DIR" ]; then
        "$BIN_PATH" completions bash > "$BASH_COMP_DIR/bit-pdf" 2>/dev/null || true
        "$BIN_PATH" completions bash > "$BASH_COMP_DIR/pdfx" 2>/dev/null || true
    fi
    if [ -d "$FISH_COMP_DIR" ] && [ -w "$FISH_COMP_DIR" ]; then
        "$BIN_PATH" completions fish > "$FISH_COMP_DIR/bit-pdf.fish" 2>/dev/null || true
        "$BIN_PATH" completions fish > "$FISH_COMP_DIR/pdfx.fish" 2>/dev/null || true
    fi
    if [ -d "$ZSH_COMP_DIR" ] && [ -w "$ZSH_COMP_DIR" ]; then
        "$BIN_PATH" completions zsh > "$ZSH_COMP_DIR/_bit-pdf" 2>/dev/null || true
        "$BIN_PATH" completions zsh > "$ZSH_COMP_DIR/_pdfx" 2>/dev/null || true
    fi
fi

# 8. Check PATH configuration
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo ""
        warn "Directory '$INSTALL_DIR' is not in your system PATH."
        warn "Add it to your profile (~/.bashrc, ~/.zshrc, etc.):"
        echo -e "    ${CYAN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}\n"
        ;;
esac

echo ""
success "Bit PDF Reader installation complete! 🎉"
echo -e "Try running:"
echo -e "  ${BOLD}bit-pdf --help${NC}   (or the shorthand ${BOLD}pdfx${NC})"
echo -e "  ${BOLD}bit-pdf open file.pdf${NC} (interactive dual-pane TUI)"
echo -e "\nSource Repository: ${REPO_URL}"
