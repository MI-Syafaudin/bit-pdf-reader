use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "noble-pdf",
    bin_name = "noble-pdf",
    author = "Syafudin & Noble PDF Reader Contributors",
    version = "0.1.0",
    about = "Noble PDF Reader — High-Performance Terminal & CLI PDF Swiss Army Knife for Linux",
    long_about = "Noble PDF Reader provides robust PDF operations including merging, splitting, extraction, rotation, text extraction, OCR, compression, and an interactive TUI."
)]
pub struct Cli {
    #[arg(short, long, global = true, help = "Enable verbose debug logging")]
    pub debug: bool,

    #[arg(
        long,
        global = true,
        help = "Output result in machine-readable JSON format"
    )]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Open interactive PDF TUI viewer")]
    Open {
        #[arg(help = "Path to the PDF file to open")]
        file: Option<PathBuf>,
    },

    #[command(about = "Inspect document metadata, version, encryption, and page dimensions")]
    Info {
        #[arg(required = true, help = "Path to the PDF file")]
        file: PathBuf,
    },

    #[command(about = "Merge multiple PDF files into a single document")]
    Merge {
        #[arg(required = true, num_args = 2.., help = "List of PDF files to merge (at least 2)")]
        files: Vec<PathBuf>,

        #[arg(short, long, required = true, help = "Target destination output file")]
        output: PathBuf,
    },

    #[command(about = "Split a PDF document into page ranges or individual parts")]
    Split {
        #[arg(required = true, help = "Path to the PDF file to split")]
        file: PathBuf,

        #[arg(
            short,
            long,
            required = true,
            help = "Page range (e.g. '1-5', '1,3,5', 'odd', 'even')"
        )]
        pages: String,

        #[arg(
            short,
            long,
            help = "Destination directory for split files (defaults to current dir)"
        )]
        output_dir: Option<PathBuf>,
    },

    #[command(about = "Extract specific pages from a PDF into a new document")]
    Extract {
        #[arg(required = true, help = "Path to the PDF file")]
        file: PathBuf,

        #[arg(
            short,
            long,
            required = true,
            help = "Page range to extract (e.g. '1-3', '5,8')"
        )]
        pages: String,

        #[arg(short, long, required = true, help = "Target output PDF file")]
        output: PathBuf,
    },

    #[command(about = "Rotate pages in a PDF document")]
    Rotate {
        #[arg(required = true, help = "Path to the PDF file")]
        file: PathBuf,

        #[arg(
            short,
            long,
            default_value = "all",
            help = "Page range to rotate ('all', '1-5', 'odd')"
        )]
        pages: String,

        #[arg(
            short = 'a',
            long = "degrees",
            default_value_t = 90,
            help = "Rotation angle in degrees: 90, 180, or 270"
        )]
        degrees: u16,

        #[arg(short, long, required = true, help = "Target output PDF file")]
        output: PathBuf,
    },

    #[command(about = "Delete specific pages from a PDF document")]
    Delete {
        #[arg(required = true, help = "Path to the PDF file")]
        file: PathBuf,

        #[arg(
            short,
            long,
            required = true,
            help = "Page range to delete (e.g. '1,3', '5-10')"
        )]
        pages: String,

        #[arg(short, long, required = true, help = "Target output PDF file")]
        output: PathBuf,

        #[arg(short, long, help = "Confirm deletion without interactive prompt")]
        yes: bool,
    },

    #[command(about = "Reorder pages in a PDF document")]
    Reorder {
        #[arg(required = true, help = "Path to the PDF file")]
        file: PathBuf,

        #[arg(
            short = 's',
            long,
            required = true,
            help = "New page order as comma-separated 1-indexed numbers (e.g. '3,1,2,4')"
        )]
        order: String,

        #[arg(short, long, required = true, help = "Target output PDF file")]
        output: PathBuf,
    },

    #[command(about = "Extract text content from a PDF document")]
    Text {
        #[arg(required = true, help = "Path to the PDF file")]
        file: PathBuf,

        #[arg(
            short,
            long,
            default_value = "all",
            help = "Page range to extract ('all', '1-5', 'odd')"
        )]
        pages: String,

        #[arg(
            short,
            long,
            default_value = "plain",
            help = "Output text format: 'plain', 'json', or 'jsonl'"
        )]
        format: String,

        #[arg(
            short,
            long,
            help = "Destination file for text (prints to stdout if omitted)"
        )]
        output: Option<PathBuf>,
    },

    #[command(about = "Optimize and compress PDF document streams")]
    Compress {
        #[arg(required = true, help = "Path to the PDF file to compress")]
        file: PathBuf,

        #[arg(short, long, required = true, help = "Target compressed output file")]
        output: PathBuf,
    },

    #[command(about = "Run OCR on scanned PDF pages to generate searchable document")]
    Ocr {
        #[arg(required = true, help = "Path to the PDF file")]
        file: PathBuf,

        #[arg(
            short,
            long,
            default_value = "eng",
            help = "OCR language (e.g. 'eng', 'ind', 'eng+ind')"
        )]
        lang: String,

        #[arg(
            short,
            long,
            default_value = "all",
            help = "Page range to perform OCR on"
        )]
        pages: String,

        #[arg(short, long, required = true, help = "Target output searchable PDF")]
        output: PathBuf,
    },

    #[command(about = "Clean recent files and job history cache")]
    Clean {
        #[arg(long, help = "Clear recent opened files history")]
        history: bool,

        #[arg(long, help = "Clear cached metadata and temporary files")]
        cache: bool,

        #[arg(long, help = "Clear all recent files, history, and cache")]
        all: bool,
    },

    #[command(about = "Generate shell completion scripts (bash, zsh, fish, powershell)")]
    Completions {
        #[arg(value_enum, help = "Target shell")]
        shell: clap_complete::Shell,
    },
}
