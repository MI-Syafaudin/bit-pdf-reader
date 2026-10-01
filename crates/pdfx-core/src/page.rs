use crate::errors::PdfxError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageOrientation {
    Portrait,
    Landscape,
    Square,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PageDimensions {
    pub width: f32,
    pub height: f32,
    pub rotation: u16,
}

impl PageDimensions {
    pub fn new(width: f32, height: f32, rotation: u16) -> Self {
        Self {
            width,
            height,
            rotation: rotation % 360,
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        if self.height <= 0.0 {
            1.0
        } else {
            self.width / self.height
        }
    }

    pub fn orientation(&self) -> PageOrientation {
        let effective_width = if self.rotation == 90 || self.rotation == 270 {
            self.height
        } else {
            self.width
        };
        let effective_height = if self.rotation == 90 || self.rotation == 270 {
            self.width
        } else {
            self.height
        };

        if (effective_width - effective_height).abs() < 1.0 {
            PageOrientation::Square
        } else if effective_width > effective_height {
            PageOrientation::Landscape
        } else {
            PageOrientation::Portrait
        }
    }
}

/// Flexible specification for selecting pages (1-indexed)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageRange {
    All,
    Single(usize),
    Range(usize, usize),
    Odd,
    Even,
    Composite(Vec<PageRange>),
}

impl PageRange {
    /// Parse a page range expression like:
    /// - "all"
    /// - "odd"
    /// - "even"
    /// - "1-5"
    /// - "1,3,5"
    /// - "1-5,8,11-15"
    pub fn parse(input: &str) -> Result<Self, PdfxError> {
        let trimmed = input.trim().to_lowercase();
        if trimmed.is_empty() || trimmed == "all" {
            return Ok(PageRange::All);
        }
        if trimmed == "odd" {
            return Ok(PageRange::Odd);
        }
        if trimmed == "even" {
            return Ok(PageRange::Even);
        }

        let parts: Vec<&str> = trimmed
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if parts.is_empty() {
            return Err(PdfxError::InvalidPageRange {
                input: input.to_string(),
                reason: "Empty or invalid page range expression".to_string(),
            });
        }

        let mut ranges = Vec::new();
        for part in parts {
            if let Some((start_s, end_s)) = part.split_once('-') {
                let start: usize =
                    start_s
                        .trim()
                        .parse()
                        .map_err(|_| PdfxError::InvalidPageRange {
                            input: input.to_string(),
                            reason: format!("Invalid start page number '{}'", start_s),
                        })?;
                let end: usize = end_s
                    .trim()
                    .parse()
                    .map_err(|_| PdfxError::InvalidPageRange {
                        input: input.to_string(),
                        reason: format!("Invalid end page number '{}'", end_s),
                    })?;

                if start == 0 || end == 0 {
                    return Err(PdfxError::InvalidPageRange {
                        input: input.to_string(),
                        reason: "Page numbers are 1-indexed and must be >= 1".to_string(),
                    });
                }
                if start > end {
                    return Err(PdfxError::InvalidPageRange {
                        input: input.to_string(),
                        reason: format!("Start page {} is greater than end page {}", start, end),
                    });
                }
                ranges.push(PageRange::Range(start, end));
            } else {
                let page: usize = part.parse().map_err(|_| PdfxError::InvalidPageRange {
                    input: input.to_string(),
                    reason: format!("Invalid page number '{}'", part),
                })?;

                if page == 0 {
                    return Err(PdfxError::InvalidPageRange {
                        input: input.to_string(),
                        reason: "Page numbers are 1-indexed and must be >= 1".to_string(),
                    });
                }
                ranges.push(PageRange::Single(page));
            }
        }

        if ranges.len() == 1 {
            Ok(ranges.remove(0))
        } else {
            Ok(PageRange::Composite(ranges))
        }
    }

    /// Resolves the page range to a deduplicated, sorted list of 1-indexed page numbers.
    pub fn resolve(&self, total_pages: usize) -> Result<Vec<usize>, PdfxError> {
        if total_pages == 0 {
            return Ok(Vec::new());
        }

        let mut pages = Vec::new();
        self.collect_pages(total_pages, &mut pages)?;
        pages.sort_unstable();
        pages.dedup();
        Ok(pages)
    }

    /// Computes the complement (remaining pages) when deleting the pages in this range.
    pub fn complement(&self, total_pages: usize) -> Result<Vec<usize>, PdfxError> {
        let to_remove = self.resolve(total_pages)?;
        let to_remove_set: std::collections::HashSet<usize> = to_remove.into_iter().collect();
        let remaining: Vec<usize> = (1..=total_pages)
            .filter(|p| !to_remove_set.contains(p))
            .collect();
        Ok(remaining)
    }

    fn collect_pages(&self, total_pages: usize, acc: &mut Vec<usize>) -> Result<(), PdfxError> {
        match self {
            PageRange::All => {
                acc.extend(1..=total_pages);
            }
            PageRange::Odd => {
                acc.extend((1..=total_pages).filter(|p| p % 2 != 0));
            }
            PageRange::Even => {
                acc.extend((1..=total_pages).filter(|p| p % 2 == 0));
            }
            PageRange::Single(p) => {
                if *p > total_pages {
                    return Err(PdfxError::PageOutOfBounds {
                        index: *p,
                        total: total_pages,
                    });
                }
                acc.push(*p);
            }
            PageRange::Range(start, end) => {
                if *end > total_pages {
                    return Err(PdfxError::PageOutOfBounds {
                        index: *end,
                        total: total_pages,
                    });
                }
                acc.extend(*start..=*end);
            }
            PageRange::Composite(sub_ranges) => {
                for r in sub_ranges {
                    r.collect_pages(total_pages, acc)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_dimensions_orientation() {
        let portrait = PageDimensions::new(595.0, 842.0, 0); // A4 portrait
        assert_eq!(portrait.orientation(), PageOrientation::Portrait);

        let rotated = PageDimensions::new(595.0, 842.0, 90);
        assert_eq!(rotated.orientation(), PageOrientation::Landscape);

        let square = PageDimensions::new(500.0, 500.0, 0);
        assert_eq!(square.orientation(), PageOrientation::Square);
    }

    #[test]
    fn test_page_range_parsing() {
        assert_eq!(PageRange::parse("all").unwrap(), PageRange::All);
        assert_eq!(PageRange::parse("odd").unwrap(), PageRange::Odd);
        assert_eq!(PageRange::parse("even").unwrap(), PageRange::Even);
        assert_eq!(PageRange::parse("5").unwrap(), PageRange::Single(5));
        assert_eq!(PageRange::parse("1-10").unwrap(), PageRange::Range(1, 10));

        let composite = PageRange::parse("1-3, 5, 8-10").unwrap();
        match composite {
            PageRange::Composite(parts) => {
                assert_eq!(parts.len(), 3);
                assert_eq!(parts[0], PageRange::Range(1, 3));
                assert_eq!(parts[1], PageRange::Single(5));
                assert_eq!(parts[2], PageRange::Range(8, 10));
            }
            _ => panic!("Expected Composite range"),
        }
    }

    #[test]
    fn test_page_range_resolve() {
        let pr = PageRange::parse("1-3, 5, 2-4").unwrap();
        let pages = pr.resolve(10).unwrap();
        assert_eq!(pages, vec![1, 2, 3, 4, 5]);

        let odd = PageRange::Odd.resolve(5).unwrap();
        assert_eq!(odd, vec![1, 3, 5]);

        let even = PageRange::Even.resolve(5).unwrap();
        assert_eq!(even, vec![2, 4]);
    }

    #[test]
    fn test_page_range_complement() {
        let to_delete = PageRange::parse("2, 4").unwrap();
        let remaining = to_delete.complement(5).unwrap();
        assert_eq!(remaining, vec![1, 3, 5]);
    }

    #[test]
    fn test_page_range_errors() {
        assert!(PageRange::parse("0").is_err());
        assert!(PageRange::parse("10-5").is_err());
        assert!(PageRange::parse("abc").is_err());

        let out_of_bounds = PageRange::Single(15);
        assert!(out_of_bounds.resolve(10).is_err());
    }
}
