# PDFX 📄⚡

> **High-Performance Terminal & CLI PDF Management and Processing Swiss Army Knife for Linux**

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Rust: 1.70+](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Platform: Linux](https://img.shields.io/badge/platform-Linux-lightgrey.svg)](https://kernel.org)
[![Zero GUI Dependencies](https://img.shields.io/badge/gui-zero_dependencies-brightgreen.svg)]()

PDFX is a blazingly fast, modern Rust-native PDF processor and viewer built specifically for Linux environments (CachyOS, Arch Linux, Ubuntu, Debian, Fedora, openSUSE). It provides a **Dual-Mode Interface**:

1. **Interactive TUI**: A dual-pane terminal viewer powered by [Ratatui](https://ratatui.rs) and [Crossterm](https://github.com/crossterm-rs/crossterm) with 24-bit TrueColor half-block rendering, fast text reading, fuzzy file browsing, bookmarks, and in-document search.
2. **Scriptable CLI**: A POSIX-compliant command-line utility with atomic operations, structured JSON output (`--json`), and predictable exit codes designed for piping and shell automation.

---

## 🚀 Key Features

- **Dual-Mode Experience**: Launch the intuitive keyboard-driven TUI or automate workflows via high-speed CLI subcommands.
- **Pure-Rust Structural Engine**: Blazing fast merge, split, extract, rotate, delete, reorder, and compression powered by `lopdf`.
- **Dynamic Rendering Fallback**: Visual rasterization using 24-bit TrueColor half-block (`▀`) characters with seamless fallback across terminal environments.
- **Asynchronous OCR Pipeline**: Background OCR worker with progress channels and cancellation tokens powered by Tesseract and image binarization.
- **Bounded Memory Guarantees**: Strict LRU-cached page store with configurable byte budgets (zero memory leaks under heavy churn).
- **Atomic File Safety**: All output writes utilize atomic staging (`tempfile` + `persist`), preventing corrupted files on unexpected interrupts.
- **Persistent Local State**: SQLite database with Write-Ahead Logging (`WAL`) mode for bookmarks, recent files, and operation logs stored in `~/.local/share/pdfx/pdfx.db`.
- **POSIX Standard Exit Codes**: Predictable return codes (0-7) for clean integration into CI/CD pipelines, bash scripts, and headless daemons.

---

## 🏗️ Architecture

```
                          ┌───────────────────────────┐
                          │         pdfx CLI          │
                          │   (Clap 4.5 Subcommands)  │
                          └─────────────┬─────────────┘
                                        │
           ┌────────────────────────────┼────────────────────────────┐
           │                            │                            │
           ▼                            ▼                            ▼
┌─────────────────────┐      ┌─────────────────────┐      ┌─────────────────────┐
│      pdfx-tui       │      │     pdfx-engine     │      │      pdfx-ocr       │
│ • Split-pane Reader │      │ • Pure-Rust lopdf   │      │ • Tesseract Runner  │
│ • TrueColor Half-Blk│◄────►│ • Dynamic Pdfium    │◄────►│ • Binarization      │
│ • Async Event Loop  │      │ • Atomic Saves      │      │ • Cancellation Token│
└──────────┬──────────┘      └──────────┬──────────┘      └──────────┬──────────┘
           │                            │                            │
           └────────────────────────────┼────────────────────────────┘
                                        ▼
                             ┌─────────────────────┐
                             │    pdfx-storage     │
                             │ • SQLite (WAL Mode) │
                             │ • LRU Memory Cache  │
                             │ • Bookmarks & State │
                             └──────────┬──────────┘
                                        ▼
                             ┌─────────────────────┐
                             │      pdfx-core      │
                             │ • Domain Types      │
                             │ • Error Hierarchy   │
                             │ • Range Parsers     │
                             └─────────────────────┘
```

---

## 📦 Installation

### Arch Linux / CachyOS

Using the included PKGBUILD:
```bash
git clone https://github.com/pdfx/pdfx.git
cd pdfx/dist/arch
makepkg -si
```

Or using an AUR helper:
```bash
paru -S pdfx
# or
yay -S pdfx
```

### Ubuntu / Debian

Build and install `.deb` package:
```bash
git clone https://github.com/pdfx/pdfx.git
cd pdfx
dpkg-buildpackage -us -uc -b
sudo dpkg -i ../pdfx_*.deb
```

### Fedora / RHEL

Build using RPM spec:
```bash
git clone https://github.com/pdfx/pdfx.git
cd pdfx
rpmbuild -ba dist/fedora/pdfx.spec
```

### Via Cargo (Any Linux)

Ensure you have Rust 1.70+ installed:
```bash
cargo install --locked --path pdfx-cli
```

### Optional Dependencies (OCR)

To enable optical character recognition for scanned PDFs:
- **Arch / CachyOS**: `sudo pacman -S tesseract tesseract-data-eng tesseract-data-ind`
- **Ubuntu / Debian**: `sudo apt install tesseract-ocr tesseract-ocr-eng tesseract-ocr-ind`
- **Fedora**: `sudo dnf install tesseract tesseract-langpack-eng tesseract-langpack-ind`

---

## 🖥️ TUI Navigation & Keybindings

Launch the interactive reader:
```bash
pdfx open document.pdf
# or simply launch the dashboard:
pdfx
```

| Key | Action |
|:---|:---|
| `j` / `↓` | Scroll text down / Next item |
| `k` / `↑` | Scroll text up / Previous item |
| `h` / `l` | Previous page / Next page |
| `PgDn` / `Space` | Jump forward one page |
| `PgUp` / `Shift+Space` | Jump backward one page |
| `Home` / `End` | Jump to first / last page |
| `Tab` | Switch focus between panes and tabs |
| `g` | Open **Jump to Page** dialog |
| `/` | In-document search (highlights matches in yellow) |
| `n` / `N` | Next / previous search match |
| `b` | Toggle bookmark on current page |
| `f` | Open interactive file picker dialog |
| `i` | Show document metadata modal |
| `?` | Show interactive help screen |
| `Esc` | Close active dialog / dismiss search |
| `q` | Quit application |

---

## ⌨️ CLI Usage & Command Reference

### 1. Document Information
Display metadata, page count, PDF version, encryption, and page dimensions:
```bash
pdfx info document.pdf
pdfx info document.pdf --json
```

### 2. Merging PDFs
Combine multiple PDF files into a single document:
```bash
pdfx merge cover.pdf chapter1.pdf chapter2.pdf -o full_book.pdf
```

### 3. Splitting PDFs
Split a document into individual pages or bursts of $N$ pages:
```bash
# Split into single pages: doc-page-001.pdf, doc-page-002.pdf, ...
pdfx split document.pdf -o ./output_dir/

# Split into chunks of 5 pages:
pdfx split document.pdf --chunk-size 5 -o ./chunks/
```

### 4. Page Extraction
Extract specific pages or ranges using standard range syntax:
```bash
pdfx extract document.pdf -p "1-3,5,10-15" -o extracted.pdf
```

### 5. Page Rotation
Rotate all or specific pages clockwise (90, 180, 270 degrees):
```bash
# Rotate entire document 90 degrees
pdfx rotate document.pdf -a 90 -o rotated.pdf

# Rotate only pages 2 and 4 by 180 degrees
pdfx rotate document.pdf -a 180 -p "2,4" -o rotated.pdf
```

### 6. Page Deletion
Remove unwanted pages from a document:
```bash
pdfx delete document.pdf -p "2,5-7" -o clean.pdf
```

### 7. Page Reordering
Arbitrarily reorder or duplicate pages:
```bash
pdfx reorder document.pdf -s "3,2,1,4-10" -o reordered.pdf
```

### 8. Text Extraction
Extract plain text from document pages to stdout or a text file:
```bash
# Dump text to stdout
pdfx text document.pdf

# Extract text from specific pages to file
pdfx text document.pdf -p "1-5" -o output.txt
```

### 9. Compression & Optimization
Remove unused objects, duplicate streams, and compress content streams:
```bash
pdfx compress heavy.pdf -o optimized.pdf
```

### 10. OCR (Optical Character Recognition)
Recognize text in scanned documents or generate searchable sandwich PDFs:
```bash
# Extract text via OCR
pdfx ocr scanned.pdf -l eng -o text

# Produce a searchable PDF
pdfx ocr scanned.pdf -l eng+ind -o searchable.pdf
```

### 11. Cache & History Management
Inspect and clear local database caches and temporary files:
```bash
# Inspect storage footprint
pdfx clean --stats

# Wipe recent files, bookmarks, and staging residue
pdfx clean --all
```

### 12. Shell Completions
Generate tab-completion scripts for your active shell:
```bash
# Bash
source <(pdfx completions bash)

# Zsh
pdfx completions zsh > "${fpath[1]}/_pdfx"

# Fish
pdfx completions fish > ~/.config/fish/completions/pdfx.fish
```

---

## 🚦 POSIX Exit Codes

PDFX adheres strictly to POSIX process exit conventions for deterministic error handling:

| Exit Code | Constant | Meaning |
|:---:|:---|:---|
| `0` | `SUCCESS` | Operation completed successfully |
| `1` | `GENERAL_ERROR` | Unclassified runtime error |
| `2` | `INVALID_ARGS` | Invalid CLI options, range syntax, or missing arguments |
| `3` | `INPUT_ERROR` | File not found, permission denied, or unreadable source file |
| `4` | `OUTPUT_ERROR` | Target directory unwritable or disk full |
| `5` | `PDF_ERROR` | Corrupted, password-protected, or unparseable PDF stream |
| `6` | `OCR_ERROR` | Tesseract binary missing, language pack absent, or OCR failed |
| `7` | `CANCELLED` | Operation cancelled by user signal (SIGINT / Ctrl+C) |

---

## ⚙️ Configuration & Storage

PDFX respects standard XDG directory specifications:

| Resource | Path | Description |
|:---|:---|:---|
| **Database** | `~/.local/share/pdfx/pdfx.db` | SQLite WAL database for bookmarks, recent files, job history |
| **Config** | `~/.config/pdfx/config.toml` | User preferences and keybindings |
| **Cache** | `~/.cache/pdfx/` | Temporary OCR renders and raster tiles |

---

## 🧪 Testing & Verification

The PDFX workspace contains 79+ comprehensive unit and end-to-end integration tests:

```bash
# Run all tests across all crates
cargo test --workspace

# Run with address sanitizer or memory benchmarks
cargo test --package pdfx-cli --test memory_profiling -- --nocapture

# Run clippy with strict zero-warning enforcement
cargo clippy --workspace --all-targets -- -D warnings

# Verify formatting
cargo fmt --check
```

---

## 📄 License

PDFX is distributed under the terms of the **Apache License 2.0**.
See [LICENSE](LICENSE) for full details.
