use lopdf::{dictionary, Dictionary, Document, Object, Stream};
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn create_sample_pdf(path: &Path, pages_count: usize) -> std::io::Result<()> {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    });

    let mut page_ids = Vec::new();
    for i in 1..=pages_count {
        let text = format!("BT /F1 14 Tf 72 700 Td (Process Exec Page {}) Tj ET", i);
        let content_id = doc.add_object(Object::Stream(Stream::new(
            Dictionary::new(),
            text.into_bytes(),
        )));

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        page_ids.push(page_id);
    }

    let pages_dict = dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids.into_iter().map(Object::Reference).collect::<Vec<_>>(),
        "Count" => pages_count as u32,
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    doc.save(path)?;
    Ok(())
}

fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_pdfx")
}

#[test]
fn test_binary_help_and_version() {
    let output = Command::new(bin_path()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Noble PDF Reader"));
    assert!(stdout.contains("Commands:"));

    let ver_output = Command::new(bin_path()).arg("--version").output().unwrap();
    assert!(ver_output.status.success());
    let ver_stdout = String::from_utf8_lossy(&ver_output.stdout);
    assert!(ver_stdout.contains("0.1.0"));
}

#[test]
fn test_binary_missing_file_exit_code_3() {
    let output = Command::new(bin_path())
        .args(["info", "/path/to/missing_dummy_document.pdf"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Input file not found"));

    // JSON mode error
    let json_output = Command::new(bin_path())
        .args(["--json", "info", "/path/to/missing_dummy_document.pdf"])
        .output()
        .unwrap();

    assert_eq!(json_output.status.code(), Some(3));
    let json_stderr = String::from_utf8_lossy(&json_output.stderr);
    assert!(json_stderr.contains("\"status\": \"error\""));
    assert!(json_stderr.contains("\"command\": \"info\""));
}

#[test]
fn test_binary_invalid_args_exit_code_2() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("sample.pdf");
    create_sample_pdf(&pdf, 2).unwrap();
    let dest = dir.path().join("out.pdf");

    // Invalid rotation degrees 45
    let output = Command::new(bin_path())
        .args([
            "rotate",
            pdf.to_str().unwrap(),
            "-a",
            "45",
            "-o",
            dest.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_binary_text_stdout_and_json() {
    let dir = tempdir().unwrap();
    let pdf = dir.path().join("test_text.pdf");
    create_sample_pdf(&pdf, 2).unwrap();

    // Plain text to stdout
    let output = Command::new(bin_path())
        .args(["text", pdf.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Process Exec Page 1"));
    assert!(stdout.contains("Process Exec Page 2"));

    // JSON format to stdout
    let json_output = Command::new(bin_path())
        .args(["text", pdf.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert!(json_output.status.success());
    let json_stdout = String::from_utf8_lossy(&json_output.stdout);
    assert!(json_stdout.contains("\"pages_extracted\": 2"));
}

#[test]
fn test_binary_clean_json_output() {
    let output = Command::new(bin_path())
        .args(["clean", "--all", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\": \"success\""));
    assert!(stdout.contains("\"command\": \"clean\""));
    assert!(stdout.contains("\"history_cleared\": true"));
}
