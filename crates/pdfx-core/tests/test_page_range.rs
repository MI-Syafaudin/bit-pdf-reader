use pdfx_core::page::PageRange;

#[test]
fn test_complex_page_ranges() {
    let pr = PageRange::parse("1-3, 5, 7-9, 12").unwrap();
    let resolved = pr.resolve(15).unwrap();
    assert_eq!(resolved, vec![1, 2, 3, 5, 7, 8, 9, 12]);
}

#[test]
fn test_odd_and_even_resolution() {
    let odd = PageRange::parse("odd").unwrap();
    assert_eq!(odd.resolve(6).unwrap(), vec![1, 3, 5]);

    let even = PageRange::parse("even").unwrap();
    assert_eq!(even.resolve(6).unwrap(), vec![2, 4, 6]);
}

#[test]
fn test_deduplication_and_sorting() {
    let pr = PageRange::parse("5, 1-3, 2, 4").unwrap();
    let resolved = pr.resolve(10).unwrap();
    assert_eq!(resolved, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_all_pages_resolution() {
    let all1 = PageRange::parse("all").unwrap();
    assert_eq!(all1.resolve(4).unwrap(), vec![1, 2, 3, 4]);

    let all2 = PageRange::parse("").unwrap();
    assert_eq!(all2.resolve(4).unwrap(), vec![1, 2, 3, 4]);
}

#[test]
fn test_out_of_bounds_rejection() {
    let pr = PageRange::parse("1-10").unwrap();
    assert!(pr.resolve(5).is_err());

    let single = PageRange::parse("10").unwrap();
    assert!(single.resolve(9).is_err());
}

#[test]
fn test_invalid_syntax_rejection() {
    assert!(PageRange::parse("0").is_err());
    assert!(PageRange::parse("0-5").is_err());
    assert!(PageRange::parse("5-2").is_err());
    assert!(PageRange::parse("abc").is_err());
    assert!(PageRange::parse("1-foo").is_err());
}

#[test]
fn test_page_range_complement() {
    let pr = PageRange::parse("2, 4, 6").unwrap();
    let remaining = pr.complement(7).unwrap();
    assert_eq!(remaining, vec![1, 3, 5, 7]);
}
