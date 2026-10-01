pub mod cancellation;
pub mod document;
pub mod errors;
pub mod jobs;
pub mod metadata;
pub mod operations;
pub mod page;
pub mod render;
pub mod search;

pub use cancellation::CancellationToken;
pub use document::{DocumentSummary, OutlineItem};
pub use errors::PdfxError;
pub use jobs::{ActiveJob, JobInfo, JobProgress, JobStatus};
pub use metadata::{DocumentMetadata, DocumentPermissions};
pub use operations::{
    CompressOperation, CompressProfile, DeleteOperation, ExtractOperation, ImageExportFormat,
    MergeOperation, OcrOperation, OperationProgress, RenderExportOperation, ReorderOperation,
    RotateOperation, SplitOperation, TextExtractOperation, TextFormat,
};
pub use page::{PageDimensions, PageOrientation, PageRange};
pub use render::{FitMode, RenderOptions, RenderedBitmap, TextPageLayout, TextSpan};
pub use search::{SearchMatch, SearchQuery};
