pub mod args;
pub mod commands;
pub mod output;

use clap::Parser;
use crate::args::{Cli, Commands};
use crate::commands::*;
use crate::output::print_json_error;
use pdfx_core::errors::PdfxError;
use std::process::exit;

pub fn exit_code_for_error(err: &PdfxError) -> i32 {
    match err {
        PdfxError::InvalidPageRange { .. }
        | PdfxError::PageOutOfBounds { .. }
        | PdfxError::InvalidRotation { .. }
        | PdfxError::InvalidOperation { .. }
        | PdfxError::Config(_) => 2,

        PdfxError::IoError(msg) => {
            let lower = msg.to_lowercase();
            if lower.contains("not found") || lower.contains("no such file") {
                3
            } else if lower.contains("permission")
                || lower.contains("read-only")
                || lower.contains("cannot write")
            {
                4
            } else {
                1
            }
        }

        PdfxError::CorruptedDocument { .. }
        | PdfxError::PasswordRequired(_)
        | PdfxError::InvalidPassword
        | PdfxError::Engine(_) => 5,

        PdfxError::Ocr(_) => 6,
        PdfxError::Cancelled => 7,
        PdfxError::Database(_) => 1,
    }
}

pub fn run_cli() {
    let cli = Cli::parse();

    if cli.debug {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::WARN)
            .init();
    }

    let json = cli.json;

    let res = match cli.command {
        Some(Commands::Open { file }) => ("open", execute_open(file)),
        Some(Commands::Info { file }) => ("info", execute_info(&file, json)),
        Some(Commands::Merge { files, output }) => ("merge", execute_merge(&files, &output, json)),
        Some(Commands::Split {
            file,
            pages,
            output_dir,
        }) => (
            "split",
            execute_split(&file, &pages, output_dir.as_deref(), json),
        ),
        Some(Commands::Extract {
            file,
            pages,
            output,
        }) => ("extract", execute_extract(&file, &pages, &output, json)),
        Some(Commands::Rotate {
            file,
            pages,
            degrees,
            output,
        }) => (
            "rotate",
            execute_rotate(&file, &pages, degrees, &output, json),
        ),
        Some(Commands::Delete {
            file,
            pages,
            output,
            yes,
        }) => ("delete", execute_delete(&file, &pages, &output, yes, json)),
        Some(Commands::Reorder {
            file,
            order,
            output,
        }) => ("reorder", execute_reorder(&file, &order, &output, json)),
        Some(Commands::Text {
            file,
            pages,
            format,
            output,
        }) => (
            "text",
            execute_text(&file, &pages, &format, output.as_deref(), json),
        ),
        Some(Commands::Compress { file, output }) => {
            ("compress", execute_compress(&file, &output, json))
        }
        Some(Commands::Ocr {
            file,
            lang,
            pages,
            output,
        }) => ("ocr", execute_ocr(&file, &lang, &pages, &output, json)),
        Some(Commands::Clean {
            history,
            cache,
            all,
        }) => ("clean", execute_clean(history, cache, all, json)),
        Some(Commands::Completions { shell }) => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "noble-pdf", &mut std::io::stdout());
            ("completions", Ok(()))
        }
        None => {
            // Default to launching interactive TUI
            ("open", execute_open(None))
        }
    };

    if let (cmd_name, Err(err)) = res {
        if json {
            print_json_error(cmd_name, &err.to_string());
        } else {
            eprintln!("Error: {}", err);
        }
        exit(exit_code_for_error(&err));
    }
}
