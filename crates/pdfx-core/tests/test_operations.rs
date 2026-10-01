use pdfx_core::operations::{
    ExtractOperation, MergeOperation, OperationProgress, ReorderOperation, RotateOperation,
};
use pdfx_core::page::PageRange;
use std::path::PathBuf;

#[test]
fn test_merge_validation_rules() {
    let valid_merge = MergeOperation::new(
        vec![PathBuf::from("a.pdf"), PathBuf::from("b.pdf")],
        PathBuf::from("out.pdf"),
    );
    assert!(valid_merge.validate().is_ok());

    let single_file = MergeOperation::new(vec![PathBuf::from("a.pdf")], PathBuf::from("out.pdf"));
    assert!(single_file.validate().is_err());

    let overwrite_input = MergeOperation::new(
        vec![PathBuf::from("a.pdf"), PathBuf::from("b.pdf")],
        PathBuf::from("b.pdf"),
    );
    assert!(overwrite_input.validate().is_err());
}

#[test]
fn test_extract_validation() {
    let valid = ExtractOperation::new(
        PathBuf::from("doc.pdf"),
        PageRange::parse("1-3").unwrap(),
        PathBuf::from("extracted.pdf"),
    );
    assert!(valid.validate().is_ok());

    let self_overwrite = ExtractOperation::new(
        PathBuf::from("doc.pdf"),
        PageRange::parse("1-3").unwrap(),
        PathBuf::from("doc.pdf"),
    );
    assert!(self_overwrite.validate().is_err());
}

#[test]
fn test_rotate_angle_validation() {
    assert!(RotateOperation::new(
        PathBuf::from("doc.pdf"),
        PageRange::All,
        90,
        PathBuf::from("out.pdf")
    )
    .is_ok());

    assert!(RotateOperation::new(
        PathBuf::from("doc.pdf"),
        PageRange::All,
        180,
        PathBuf::from("out.pdf")
    )
    .is_ok());

    assert!(RotateOperation::new(
        PathBuf::from("doc.pdf"),
        PageRange::All,
        270,
        PathBuf::from("out.pdf")
    )
    .is_ok());

    assert!(RotateOperation::new(
        PathBuf::from("doc.pdf"),
        PageRange::All,
        360,
        PathBuf::from("out.pdf")
    )
    .is_err());

    assert!(RotateOperation::new(
        PathBuf::from("doc.pdf"),
        PageRange::All,
        45,
        PathBuf::from("out.pdf")
    )
    .is_err());
}

#[test]
fn test_reorder_validation() {
    let valid = ReorderOperation::new(
        PathBuf::from("doc.pdf"),
        vec![3, 1, 2],
        PathBuf::from("out.pdf"),
    );
    assert!(valid.is_ok());

    let empty = ReorderOperation::new(PathBuf::from("doc.pdf"), vec![], PathBuf::from("out.pdf"));
    assert!(empty.is_err());

    let has_zero = ReorderOperation::new(
        PathBuf::from("doc.pdf"),
        vec![2, 0, 1],
        PathBuf::from("out.pdf"),
    );
    assert!(has_zero.is_err());
}

#[test]
fn test_operation_progress_with_eta() {
    let p = OperationProgress::new(25, 100, "Rendering").with_eta(15);
    assert_eq!(p.percentage, 25.0);
    assert_eq!(p.eta_seconds, Some(15));
}
